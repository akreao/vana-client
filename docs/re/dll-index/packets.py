#!/usr/bin/env python3
"""The packet tables of one FFXiMain.dll build, and what changed in them between two builds.

    python3 packets.py table DIR                      writes DIR/packets.json (and packets.tsv)
    python3 packets.py diff OLD_DIR NEW_DIR [--src CHECKOUT] [--out DIR] [--queue FILE]

DIR is a dll-index work dir after ri.sh (all_decomp_full.c, funcs_full.tsv, xrefs.tsv, FFXiMain.unpacked.dll). The
diff also needs NEW_DIR/remap/ (remap.py OLD_DIR NEW_DIR).

What a table holds:
  c2s  every call of the send gate (the function that takes a c2s id, refuses one sent too soon and hands out a queue
       buffer with the id in its header): the id, the size the caller then gives the size function (header byte 1:
       (n + extra) rounded up to 4), the calling function and the call address.
  s2c  every handler registration: the id, which of the two handler tables (the dispatcher calls table "main" with
       each packet, then table "second" when the game state allows), the handler, the registering function.
The gate, the size function, the two register functions and the dispatcher are found by what their code does (the
patterns below), so a new build needs no addresses. Each call is read from Ghidra's decompile and checked against
Rizin's disassembly of the call site (the pushed id, size and handler); a function where the two disagree is listed
under "unverified" and has to be read by hand.

What the diff reports (to --out, default NEW_DIR/packet-diff/):
  report.md      everything: ids added / removed, sizes changed, senders and handlers whose code changed (remap's
                 funcmap status, and changed or new functions they call, two calls deep), our Rust code that names
                 the id or cites a changed function. Local only: it names DLL functions and addresses.
  summary.md     the part fit for a PR comment: ids, sizes, addresses, statuses and our file names. No code.
  queue-rows.md  one modlog-gaps queue row per real change; --queue FILE appends them to that queue (once per build).
Generated from the DLL: never commit any of it.
"""
import sys, os, re, json, bisect, argparse, subprocess, collections

# --- what the functions look like in Ghidra's decompile (any build) ---------------------------------------------
# the send gate: `*puVar4 = (ushort)param_1 & 0x1ff;` on the buffer it returns
GATE_RE = re.compile(r'\*\w+ = \(ushort\)param_1 & 0x1ff;')
# the size function: header = ((n + extra) / 4 rounded up) * 0x200 ^ (header & 0x1ff)
SIZE_RE = re.compile(r'\* 0x200 \^\s*\*param_1 & 0x1ff')
# a register function: table[param_1] = param_2 under a bound check of param_1 (Ghidra writes the check either way
# round: `param_1 < 0x11f` or `0x11e < param_1`), and nothing else
REG_RE = re.compile(r'\*\(undefined4 \*\)\(DAT_\w+ \+ (0x[0-9a-f]+) \+ param_1 \* 4\) = param_2;')
REG_BOUND = re.compile(r'param_1 < 0x[0-9a-f]+|0x[0-9a-f]+ < param_1')
# the dispatcher's reads of the two tables, in the order it calls them
DISPATCH_RE = re.compile(r'\*\(code \*\*\)\(DAT_\w+ \+ (0x[0-9a-f]+) \+ \(\*\w+ & 0x1ff\) \* 4\)')

LIT = re.compile(r'^(0x[0-9a-fA-F]+|\d+)$')
ADDR_NAME = re.compile(r'^(?:thunk_)?(?:FUN|LAB|SUB|DAT)_([0-9a-f]{8})$')


def lit(tok):
    tok = tok.strip()
    while tok.startswith('(') and ')' in tok and not LIT.match(tok):   # drop casts: (uint)0x10, (int)(short)x
        m = re.match(r'^\([\w\s\*]+\)(.*)$', tok)
        if not m:
            break
        tok = m.group(1).strip()
    if LIT.match(tok):
        return int(tok, 0)
    if re.match(r"^'.'$", tok):
        return ord(tok[1])
    return None


def norm(expr):
    """A size expression with Ghidra's local names blanked (they differ between builds)."""
    return re.sub(r'\b(?:[iu]Var|local_|param_|pu?c?Var|[a-z]{1,3}Var)\w*', 'v', expr).replace(' ', '')


def split_args(s, start):
    """The comma-separated arguments of the call whose '(' is at s[start]; (args, index after ')')."""
    depth, cur, args, i = 0, [], [], start
    while i < len(s):
        c = s[i]
        if c == '(':
            depth += 1
            if depth > 1:
                cur.append(c)
        elif c == ')':
            depth -= 1
            if depth == 0:
                args.append(''.join(cur).strip())
                return args, i + 1
            cur.append(c)
        elif c == ',' and depth == 1:
            args.append(''.join(cur).strip())
            cur = []
        else:
            cur.append(c)
        i += 1
    return args, i


class Build:
    """One work dir's decompile, function list and call graph."""

    def __init__(self, d):
        self.dir = d
        self.funcs = {}          # entry -> (end, name)
        self.byname = {}
        for line in open(os.path.join(d, 'funcs_full.tsv')):
            p = line.rstrip('\n').split('\t')
            if len(p) < 3 or p[0].startswith('#'):
                continue
            a, e = int(p[0], 16), int(p[1], 16)
            self.funcs[a] = (e, p[2])
            self.byname.setdefault(p[2], a)
        self.starts = sorted(self.funcs)
        self.body = {}
        cur, buf = None, []
        for line in open(os.path.join(d, 'all_decomp_full.c'), errors='replace'):
            if line.startswith('//@@FUNC '):
                if cur is not None:
                    self.body[cur] = ''.join(buf)
                cur, buf = int(line.split()[1], 16), []
            else:
                buf.append(line)
        if cur is not None:
            self.body[cur] = ''.join(buf)
        self.calls = collections.defaultdict(set)     # caller entry -> callee entries
        self.callers = collections.defaultdict(set)
        self.sites = collections.defaultdict(list)    # callee entry -> [(site, caller entry)]
        for line in open(os.path.join(d, 'xrefs.tsv')):
            p = line.rstrip('\n').split('\t')
            if len(p) < 5 or not p[4].endswith('CALL') or p[1] == '-' or p[3] == '-':
                continue
            site, fr, to = int(p[0], 16), int(p[1], 16), int(p[3], 16)
            self.calls[fr].add(to)
            self.callers[to].add(fr)
            self.sites[to].append((site, fr))
        self.version = os.path.basename(os.path.normpath(d))
        if self.version == 'base':
            self.version = os.path.basename(os.path.dirname(os.path.normpath(d)))

    def name(self, a):
        return self.funcs.get(a, (0, 'FUN_%08x' % a))[1]

    def containing(self, a):
        i = bisect.bisect_right(self.starts, a) - 1
        if i >= 0 and a <= self.funcs[self.starts[i]][0]:
            return self.starts[i]
        return None

    def resolve(self, tok):
        """A function named in the decompile -> its entry (None if it names none)."""
        tok = tok.strip().lstrip('&').strip()
        m = re.match(r'^\([\w\s\*]+\)(.*)$', tok)        # (code *)FUN_x
        if m:
            tok = m.group(1).strip()
        if tok in self.byname:
            return self.byname[tok]
        m = ADDR_NAME.match(tok)
        if m:
            return int(m.group(1), 16)
        return None


def roles(b):
    """Find the gate, the size functions, the register functions (by table) and the dispatcher's table order."""
    gate, size, reg = [], [], {}
    order = []
    for a, body in b.body.items():
        if GATE_RE.search(body) and 'param_1 < 0x' in body:
            gate.append(a)
        if SIZE_RE.search(body):
            size.append(a)
        m = REG_RE.search(body)
        if m and REG_BOUND.search(body) and body.count(';') <= 4:
            reg.setdefault(int(m.group(1), 16), []).append(a)
        offs = [int(x, 16) for x in DISPATCH_RE.findall(body)]
        if len(offs) >= 2:
            order.append((a, offs))
    if not gate or not size or len(reg) != 2 or len(order) != 1:
        raise SystemExit('%s: packet functions not found (gate %s, size %s, register tables %s, dispatchers %s): '
                         'the decompile patterns in packets.py need a look' %
                         (b.dir, gate, size, sorted('%x' % k for k in reg), [('%x' % a) for a, _ in order]))
    disp, offs = order[0]
    if set(offs[:2]) != set(reg):
        raise SystemExit('%s: dispatcher %x reads tables %s, register functions write %s' %
                         (b.dir, disp, [hex(x) for x in offs], [hex(x) for x in reg]))
    tables = {offs[0]: 'main', offs[1]: 'second'}
    return {'gate': sorted(gate), 'size': sorted(size),
            'reg': {tables[k]: sorted(v) for k, v in reg.items()},
            'table_offset': {tables[k]: k for k in reg}, 'dispatcher': disp}


def call_re(names):
    alts = '|'.join(re.escape(n) for n in sorted(names, key=len, reverse=True))
    return re.compile(r'(?<![\w])(' + alts + r')\(')


def calls(body, rx, nargs):
    """Each call `rx` matches in `body` with `nargs` arguments: (args, the variable its result goes to, text end)."""
    out = []
    for m in rx.finditer(body):
        args, end = split_args(body, m.end() - 1)
        if len(args) != nargs:
            continue
        v = re.search(r'(\w+) = (?:\([\w\s\*]+\))?$', body[max(0, m.start() - 40):m.start()])
        out.append({'args': args, 'var': v.group(1) if v else None, 'pos': m.start(), 'end': end})
    return out


def gate_calls(body, gate_c, size_c):
    """The decompile's reading of each send in `body`: candidate ids and the size given to its buffer."""
    sizes = []
    for c in calls(body, size_c, 3):
        sizes.append((c['pos'], re.sub(r'^\([\w\s\*]+\)', '', c['args'][0]).strip(), c['args'][1], c['args'][2]))
    gs = calls(body, gate_c, 3)
    out = []
    for i, g in enumerate(gs):
        a0 = g['args'][0]
        idv = lit(a0)
        ids = [idv] if idv is not None else sorted({int(x, 0) for x in re.findall(
            r'(?<![\w.])' + re.escape(a0) + r' = (0x[0-9a-fA-F]+|\d+);', body)})
        size, var, header = None, g['var'], False
        if var:
            nxt = gs[i + 1]['pos'] if i + 1 < len(gs) else len(body)
            again = re.search(r'(?<![\w.*>])' + re.escape(var) + r' = ', body[g['end']:])
            stop = min(nxt, g['end'] + again.start() if again else len(body))
            for (p, buf, n, x) in sizes:
                if g['end'] <= p < stop and buf == var:
                    nv, xv = lit(n), lit(x)
                    size = (nv + xv + 3) // 4 * 4 if nv is not None and xv is not None else \
                        'var(%s)' % norm('%s + %s' % (n, x))
                    break
            if size is None:      # the header written whole: *puVar1 = 0x60c
                h = re.search(r'\*' + re.escape(var) + r' = (0x[0-9a-fA-F]+);', body[g['end']:stop])
                if h and ids and (int(h.group(1), 16) & 0x1ff) in ids:
                    size, header = (int(h.group(1), 16) >> 9) * 4, True
        out.append({'ids': ids, 'literal': idv is not None, 'size': size, 'text': a0, 'header': header})
    return out


def reg_calls(b, body, reg_c, reg_names):
    out = []
    for c in calls(body, reg_c, 2):
        a0, a1 = c['args']
        h = 0 if a1 in ('0', '(code *)0x0', 'NULL') else b.resolve(a1)
        m = reg_c.match(body, c['pos'])
        out.append({'id': lit(a0), 'table': reg_names[m.group(1)], 'handler': h, 'text': a1})
    return out


def pair(sites, decomp, key):
    """Pair call sites (address order, each with Rizin's value or None) with the decompile's calls in that function:
    equal values first, then the rest in order. Decompile calls left over belong to code Ghidra shows inside this
    function but that another function's call site holds (fall-through fragments): dropped. (site, call or None)."""
    used, out = set(), {}
    for s, rv in sites:
        if rv is None:
            continue
        for j, d in enumerate(decomp):
            if j not in used and key(d, rv):
                used.add(j)
                out[s] = d
                break
    rest = [j for j in range(len(decomp)) if j not in used]
    for s, rv in sites:
        if s not in out and rest:
            out[s] = decomp[rest.pop(0)]
    return [(s, rv, out.get(s)) for s, rv in sites]


def extract(b, r):
    """The c2s and s2c rows, one per call site (xrefs.tsv), each read by Ghidra and by Rizin."""
    gate_names = {b.name(a) for a in r['gate']}
    size_names = {b.name(a) for a in r['size']}
    reg_names = {b.name(a): t for t, l in r['reg'].items() for a in l}
    gate_c, size_c, reg_c = call_re(gate_names), call_re(size_names), call_re(reg_names)
    own = set(r['gate']) | set(r['size']) | {a for l in r['reg'].values() for a in l}
    kinds = [(a, 'gate') for a in r['gate']] + [(a, 'size') for a in r['size']] + \
            [(a, 'reg') for l in r['reg'].values() for a in l]
    by_fn = collections.defaultdict(lambda: collections.defaultdict(list))
    for a, k in kinds:
        for site, fn in b.sites[a]:
            if fn not in own:
                by_fn[fn][k].append(site)
    dis = rizin_sites(b, [(s, fn) for fn, ks in by_fn.items() for l in ks.values() for s in l])
    c2s, s2c = [], []
    for fn, ks in sorted(by_fn.items()):
        body = b.body.get(fn, '')
        if ks['gate']:
            sites = [(s, push_args(dis[s], 1)[0] if s in dis else None) for s in sorted(ks['gate'])]
            dec = gate_calls(body, gate_c, size_c)
            pushed = collections.Counter()
            for s in ks['size']:
                if s in dis:
                    _, n, x = push_args(dis[s], 3)
                    if n is not None and x is not None:
                        pushed[(n + x + 3) // 4 * 4] += 1
            for s, rv, d in pair(sites, dec, lambda d, v: v in d['ids']):
                note = []
                if d is None:
                    ids, size = [rv], None
                    note.append('not in the decompile')
                else:
                    ids, size = d['ids'] or [rv], d['size']
                    if rv is None and d['literal']:
                        note.append('disassembly: id not a constant')
                    elif rv is not None and rv not in d['ids']:
                        note.append('decompile %s, disassembly %#x' % (d['text'], rv))
                        ids = [rv]
                    if isinstance(size, int) and not d['header']:
                        if pushed[size] > 0:
                            pushed[size] -= 1
                        else:
                            note.append('size %#x not pushed' % size)
                if ids == [None]:
                    note.append('id not read')
                for i in ids:
                    c2s.append({'id': i, 'size': size, 'by': fn, 'site': s, 'via_var': len(ids) > 1 or
                                (d is not None and not d['literal']), 'note': '; '.join(note)})
            # a size the decompile lost (Ghidra moved the buffer to another variable): one send, one push left
            left = [k for k, n in pushed.items() for _ in range(n)]
            nos = [c for c in c2s if c['by'] == fn and c['size'] is None]
            if len(left) == 1 and len({c['site'] for c in nos}) == 1 and len(ks['gate']) == 1:
                for c in nos:
                    c['size'] = left[0]
                    c['info'] = 'size from the disassembly only'
            # sends that share one size call (a goto into another case's tail): the size of this id's other sends
            for c in c2s:
                if c['by'] == fn and c['size'] is None:
                    same = {x['size'] for x in c2s if x['by'] == fn and x['id'] == c['id'] and x['size'] is not None}
                    if len(same) == 1:
                        c['size'] = same.pop()
        if ks['reg']:
            sites = [(s, tuple(push_args(dis[s], 2)) if s in dis else None) for s in sorted(ks['reg'])]
            dec = reg_calls(b, body, reg_c, reg_names)
            reg_at = {s: a for a, k in kinds if k == 'reg' for s, f in b.sites[a] if f == fn}
            tbl = {a: t for t, l in r['reg'].items() for a in l}
            for s, rv, d in pair(sites, dec, lambda d, v: (d['id'], d['handler']) == v):
                note = []
                t = tbl[reg_at[s]]
                if d is None:
                    i, h = rv if rv else (None, None)
                    note.append('not in the decompile')
                else:
                    i, h = d['id'], d['handler']
                    if d['table'] != t:
                        note.append('decompile says the %s table' % d['table'])
                    if rv is None or rv != (i, h):
                        note.append('disassembly pushes %s' % (fmt_counter(collections.Counter([rv]))
                                                               if rv else 'nothing readable'))
                s2c.append({'id': i, 'table': t, 'handler': h, 'by': fn, 'site': s, 'note': '; '.join(note)})
    return c2s, s2c


# --- Rizin: the pushed arguments at each call site ---------------------------------------------------------------
REGS = ('eax', 'ebx', 'ecx', 'edx', 'esi', 'edi', 'ebp')


def push_args(insns, n):
    """Values of the last `n` pushes before the end of `insns` (first argument first); None = not a constant."""
    vals, i = [], len(insns) - 1
    while i >= 0 and len(vals) < n:
        op = insns[i].get('disasm', '')
        if op.startswith('call '):
            break
        if op.startswith('push '):
            arg = op[5:].strip()
            if re.match(r'^(0x[0-9a-f]+|\d+)$', arg):
                vals.append(int(arg, 0))
            elif arg in REGS:
                vals.append(reg_value(insns, i, arg))
            else:
                vals.append(None)
        i -= 1
    return vals + [None] * (n - len(vals))


def reg_value(insns, i, reg):
    """The constant `reg` holds at insns[i], read back along the straight path: callee-saved registers live through
    calls, and an exit block (a ret / jmp a conditional jump skipped) is stepped over."""
    j = i - 1
    while j >= 0:
        op = insns[j].get('disasm', '')
        if op.startswith('ret') or op.startswith('jmp '):
            here = insns[j]['offset']
            k = j - 1
            while k >= 0 and not (insns[k].get('type') == 'cjmp' and insns[k].get('jump', 0) > here):
                k -= 1
            if k < 0:
                return None
            j = k - 1
            continue
        if op.startswith('call '):
            if reg in ('eax', 'ecx', 'edx'):
                return None
        elif op == 'xor %s, %s' % (reg, reg):
            return 0
        else:
            m = re.match(r'^mov %s, (0x[0-9a-f]+|\d+)$' % reg, op)
            if m:
                return int(m.group(1), 0)
            if re.match(r'^(mov|lea|pop|or|and|add|sub|movzx|movsx|inc|dec|xor|shl|shr|sar|imul|neg|not|sbb|adc|'
                        r'xchg|set\w*|cmov\w*) %s\b' % reg, op) or (reg == 'edx' and op == 'cdq'):
                return None
        j -= 1
    return None


def rizin_sites(b, sites):
    """{site: [insns up to the call]} with one Rizin run (from the function's start when it is near)."""
    dll = os.path.join(b.dir, 'FFXiMain.unpacked.dll')
    cmds, keys = [], []
    for site, fn in sorted(set(sites)):
        start = fn if 0 <= site - fn <= 0x4000 else site - 0x100   # far from its start: x86 resyncs in a few bytes
        cmds.append('pDj %d @ 0x%x' % (site - start + 5, start))
        keys.append(site)
    if not cmds:
        return {}
    script = os.path.join(b.dir, '.packets_rz.txt')
    with open(script, 'w') as f:
        f.write('\n'.join(cmds) + '\n')
    try:
        p = subprocess.run(['rizin', '-q', '-e', 'scr.color=0', '-e', 'asm.bytes=false', '-i', script, dll],
                           capture_output=True, text=True, check=True)
    finally:
        os.remove(script)
    lines = [l for l in p.stdout.splitlines() if l.startswith('[')]
    if len(lines) != len(keys):
        raise SystemExit('rizin returned %d listings for %d call sites' % (len(lines), len(keys)))
    out = {}
    for k, l in zip(keys, lines):
        insns = json.loads(l)
        # stop at the call itself: a listing from the function start may run past it
        cut = [x for x in insns if x.get('offset', 0) <= k]
        out[k] = cut[:-1] if cut and cut[-1].get('offset') == k else cut
    return out


def fmt_counter(c):
    def f(v):
        if v is None:
            return '?'
        if isinstance(v, tuple):
            return '(%s)' % ','.join(f(x) for x in v)
        return '%#x' % v
    return '[' + ' '.join(f(k) + ('x%d' % n if n > 1 else '') for k, n in sorted(c.items(), key=str)) + ']'


def table(d):
    b = Build(d)
    r = roles(b)
    c2s, s2c = extract(b, r)
    bad = collections.defaultdict(list)
    for x in c2s + s2c:
        if x['note']:
            bad['%08x' % x['by']].append('%08x %s: %s' % (x['site'], hx(x['id']), x['note']))
    t = {'build': b.version, 'roles': r,
         'c2s': sorted(c2s, key=lambda c: (c['id'] is None, c['id'] or 0, c['site'])),
         's2c': sorted(s2c, key=lambda e: (e['id'] is None, e['id'] or 0, e['table'], e['site'])),
         'unverified': dict(sorted(bad.items()))}
    with open(os.path.join(d, 'packets.json'), 'w') as f:
        json.dump(t, f, indent=1)
    with open(os.path.join(d, 'packets.tsv'), 'w') as f:
        f.write('# dir\tid\tsize / table\thandler\tfunction\tcall site\tname\tcheck\n')
        for c in t['c2s']:
            f.write('c2s\t%s\t%s\t-\t%08x\t%08x\t%s\t%s\n' % (hx(c['id']), c['size'], c['by'], c['site'],
                                                         b.name(c['by']), c['note'] or 'ok'))
        for e in t['s2c']:
            f.write('s2c\t%s\t%s\t%s\t%08x\t%08x\t%s\t%s\n' % (hx(e['id']), e['table'], hx8(e['handler']), e['by'],
                                                           e['site'], b.name(e['by']), e['note'] or 'ok'))
    return t, b


def hx(v):
    return '?' if v is None else '0x%03X' % v


def hx8(v):
    return '-' if v is None else ('0' if v == 0 else '%08x' % v)


def load_table(d):
    p = os.path.join(d, 'packets.json')
    stale = not os.path.exists(p) or any(
        os.path.getmtime(os.path.join(d, f)) > os.path.getmtime(p)
        for f in ('all_decomp_full.c', 'xrefs.tsv', 'funcs_full.tsv')) or \
        os.path.getmtime(os.path.abspath(__file__)) > os.path.getmtime(p)
    if stale:
        return table(d)
    return json.load(open(p)), Build(d)


# --- the diff ------------------------------------------------------------------------------------------------------
def read_remap(new_dir):
    rd = os.path.join(new_dir, 'remap')
    fmap, back, status, shifts = {}, {}, {}, {}
    for line in open(os.path.join(rd, 'funcmap.tsv')):
        if line.startswith('#'):
            continue
        p = line.rstrip('\n').split('\t')
        o, n = int(p[0], 16), int(p[1], 16)
        fmap[o], back[n], status[n] = n, o, p[2]
        shifts[n] = p[7] if len(p) > 7 else ''
    new = set()
    for line in open(os.path.join(rd, 'new.tsv')):
        if not line.startswith('#') and line.strip():
            new.add(int(line.split('\t')[0], 16))
    return fmap, back, status, shifts, new


def common_shifts(status, shifts):
    """Constant shifts seen in many functions: a struct that grew (noise for packets)."""
    c = collections.Counter()
    for n, s in status.items():
        if s == 'consts':
            for x in set(shifts.get(n, '').split()):
                c[x] += 1
    return {x for x, k in c.items() if k >= 20}


def fn_state(n, status, shifts, new, noise):
    if n in new:
        return 'new'
    s = status.get(n)
    if s is None:
        return 'unmatched'
    if s == 'consts' and set(shifts.get(n, '').split()) <= noise:
        return 'same'      # only the struct shift
    return s


def callee_changes(b, start, status, shifts, new, noise, depth=2, fanin=150):
    """Changed / new functions `start` reaches in `depth` calls (skipping helpers called from everywhere)."""
    out, seen, front = [], {start}, [start]
    for d in range(depth):
        nxt = []
        for f in front:
            for c in sorted(b.calls.get(f, ())):
                if c in seen or len(b.callers.get(c, ())) > fanin:
                    continue
                seen.add(c)
                st = fn_state(c, status, shifts, new, noise)
                if st not in ('same', 'unmatched'):
                    out.append((c, st, d + 1))
                nxt.append(c)
        front = nxt
    return out


DIR_WORDS = {'c2s': re.compile(r'\bc2s\b|NetCommand|Packet \{ id:|C2S_', re.I),
             's2c': re.compile(r'\bs2c\b|NetEvent|S2C_|=>', re.I)}


def src_hits(src, ids_by_dir, addrs):
    """Our Rust lines that name an id or cite an address. An id line that names this direction (c2s / s2c, a
    NetCommand / NetEvent, a match arm) is a hit, (dir, id); one that names only the other direction is left out;
    one that names neither (in packet context) goes under ('maybe', dir, id)."""
    hits = collections.defaultdict(list)
    if not src:
        return hits
    files = []
    for root, dirs, fs in os.walk(os.path.join(src, 'crates')):
        dirs[:] = [x for x in dirs if x not in ('target', 'testdata', 'vendor')]
        files += [os.path.join(root, f) for f in fs if f.endswith('.rs')]
    pats = [((dr, i), re.compile(r'\b0x%03x\b' % i, re.I)) for (dr, i) in ids_by_dir]
    apats = [(a, re.compile(r'(?:FUN_|LAB_|0x)%08x\b' % a, re.I)) for a in addrs]
    ctx = re.compile(r'packet|opcode|send', re.I)
    for path in sorted(files):
        rel = os.path.relpath(path, src)
        try:
            lines = open(path, errors='replace').read().splitlines()
        except OSError:
            continue
        net = '/net/' in rel or rel.endswith('net.rs')
        for ln, line in enumerate(lines, 1):
            if len(re.findall(r'0x[0-9a-fA-F]{6,}', line)) >= 4:
                continue      # a hex table
            at = '%s:%d' % (rel, ln)
            for (dr, i), p in pats:
                if not p.search(line):
                    continue
                other = 's2c' if dr == 'c2s' else 'c2s'
                if DIR_WORDS[dr].search(line):
                    hits[(dr, i)].append(at)
                elif not DIR_WORDS[other].search(line) and (net or ctx.search(line)):
                    hits[('maybe', dr, i)].append(at)
            for a, p in apats:
                if p.search(line):
                    hits[('addr', a)].append(at)
    return hits


def by_id(rows):
    d = collections.defaultdict(list)
    for r in rows:
        d[r['id']].append(r)
    return d


def diff(old_dir, new_dir, src, out, queue):
    to, bo = load_table(old_dir)
    tn, bn = load_table(new_dir)
    fmap, back, status, shifts, new = read_remap(new_dir)
    noise = common_shifts(status, shifts)
    unv_old = {int(k, 16) for k in to['unverified']}
    unv_new = {int(k, 16) for k in tn['unverified']}
    findings = []      # dicts: dir, id, kind, detail, addrs (new build), old addrs, verified

    def st(n):
        return fn_state(n, status, shifts, new, noise)

    # c2s
    oc, nc = by_id(to['c2s']), by_id(tn['c2s'])
    for i in sorted(set(oc) | set(nc), key=lambda x: (x is None, x or 0)):
        o, n = oc.get(i, []), nc.get(i, [])
        if i is None:
            continue
        ver = not any(r['by'] in unv_new for r in n) and not any(r['by'] in unv_old for r in o)
        if not o:
            findings.append(dict(dir='c2s', id=i, kind='new', ver=ver,
                                 detail='sent by %s, size %s' % (', '.join('%08x' % r['by'] for r in n),
                                                                 sizes(n)), addrs=[r['by'] for r in n], old=[]))
            continue
        if not n:
            findings.append(dict(dir='c2s', id=i, kind='removed', ver=ver,
                                 detail='was sent by %s' % ', '.join('%08x' % r['by'] for r in o),
                                 addrs=[], old=[r['by'] for r in o]))
            continue
        so, sn = sizes(o), sizes(n)
        if so != sn:
            findings.append(dict(dir='c2s', id=i, kind='size', ver=ver, detail='size %s -> %s' % (so, sn),
                                 addrs=[r['by'] for r in n], old=[r['by'] for r in o]))
        mo = {fmap.get(r['by']) for r in o}
        for r in n:
            s = st(r['by'])
            if r['by'] not in mo:
                findings.append(dict(dir='c2s', id=i, kind='new sender', ver=ver,
                                     detail='%08x (%s) now sends it' % (r['by'], s), addrs=[r['by']], old=[]))
            elif s not in ('same',):
                findings.append(dict(dir='c2s', id=i, kind='sender ' + s, ver=ver,
                                     detail='%08x -> %08x' % (back[r['by']], r['by']), addrs=[r['by']],
                                     old=[back[r['by']]]))
        for r in o:
            if fmap.get(r['by']) not in {x['by'] for x in n}:
                findings.append(dict(dir='c2s', id=i, kind='sender gone', ver=ver,
                                     detail='%08x no longer sends it' % r['by'], addrs=[], old=[r['by']]))

    # s2c
    oh, nh = by_id(to['s2c']), by_id(tn['s2c'])
    for i in sorted(set(oh) | set(nh), key=lambda x: (x is None, x or 0)):
        if i is None:
            continue
        o, n = oh.get(i, []), nh.get(i, [])
        ver = not any(r['by'] in unv_new for r in n) and not any(r['by'] in unv_old for r in o)
        ko = {(r['table'], r['handler']) for r in o if r['handler']}
        kn = {(r['table'], r['handler']) for r in n if r['handler']}
        if not ko and kn:
            findings.append(dict(dir='s2c', id=i, kind='new', ver=ver, detail='handler %s' % ', '.join(
                '%08x (%s table, registered by %08x)' % (r['handler'], r['table'], r['by']) for r in n if r['handler']),
                addrs=[r['handler'] for r in n if r['handler']] + [r['by'] for r in n], old=[]))
            continue
        if ko and not kn:
            findings.append(dict(dir='s2c', id=i, kind='removed', ver=ver, detail='no handler now',
                                 addrs=[], old=[h for _, h in ko]))
            continue
        mapped = {(t, fmap.get(h)) for t, h in ko}
        for t, h in sorted(kn - mapped):
            findings.append(dict(dir='s2c', id=i, kind='handler replaced', ver=ver,
                                 detail='%s table: %08x (%s)' % (t, h, st(h)), addrs=[h],
                                 old=[x for tt, x in ko if tt == t]))
        for t, h in sorted(mapped - kn):
            if h is not None and (t, h) not in kn:
                findings.append(dict(dir='s2c', id=i, kind='handler gone', ver=ver,
                                     detail='%s table: %08x' % (t, h), addrs=[], old=[back.get(h, h)]))
        for t, h in sorted(kn & mapped):
            s = st(h)
            if s != 'same':
                findings.append(dict(dir='s2c', id=i, kind='handler ' + s, ver=ver,
                                     detail='%s table: %08x -> %08x' % (t, back[h], h), addrs=[h], old=[back[h]]))
            for c, cs, dp in callee_changes(bn, h, status, shifts, new, noise):
                findings.append(dict(dir='s2c', id=i, kind='callee ' + cs, ver=ver,
                                     detail='%08x calls %08x (%s, %d deep)' % (h, c, cs, dp), addrs=[c],
                                     old=[back[c]] if c in back else []))
        reg_o = {fmap.get(r['by']) for r in o}
        for r in n:
            if r['by'] in reg_o and st(r['by']) not in ('same',) and not any(
                    f['dir'] == 's2c' and f['id'] == i and f['kind'] == 'registrar' for f in findings):
                findings.append(dict(dir='s2c', id=i, kind='registrar', ver=ver,
                                     detail='registered by %08x (%s)' % (r['by'], st(r['by'])), addrs=[r['by']],
                                     old=[back[r['by']]]))
    # c2s senders' callees too (a body builder that changed), one call deep
    for i, n in nc.items():
        if i is None:
            continue
        for r in n:
            for c, cs, dp in callee_changes(bn, r['by'], status, shifts, new, noise, depth=1):
                findings.append(dict(dir='c2s', id=i, kind='callee ' + cs, ver=True,
                                     detail='%08x calls %08x (%s)' % (r['by'], c, cs), addrs=[c],
                                     old=[back[c]] if c in back else []))

    ids = {(f['dir'], f['id']) for f in findings}
    addrs = {a for f in findings for a in f['addrs'] + f['old']}
    hits = src_hits(src, ids, addrs)
    os.makedirs(out, exist_ok=True)
    write_reports(out, to, tn, bo, bn, findings, hits, status, new)
    rows = queue_rows(tn['build'], findings, hits)
    with open(os.path.join(out, 'queue-rows.md'), 'w') as f:
        f.write(''.join('[%s] %s\n' % r for r in rows))
    if queue:
        append_queue(queue, tn['build'], rows)
    return findings


def sizes(rows):
    s = sorted({str(r['size']) if r['size'] is not None else '?' for r in rows},
               key=lambda x: (not x.isdigit(), int(x) if x.isdigit() else 0, x))
    return '/'.join(('%#x' % int(x)) if x.isdigit() else x for x in s)


# Confirmed from the tables themselves: a queue row to build.
REAL = ('new', 'removed', 'size', 'new sender', 'sender gone', 'handler replaced', 'handler gone')
# The code changed by remap's reading (instructions, not behaviour: a struct shift the compiler folded another way
# reads as "changed" too, as the 0x069 / 0x074 handlers did in 30261102_0): a queue row to read, not to build.
CHECK = ('sender changed', 'sender refs', 'sender consts', 'handler changed', 'handler refs', 'handler consts')


def group(findings):
    g = collections.OrderedDict()
    for f in findings:
        g.setdefault((f['dir'], f['id']), []).append(f)
    return g


def our_files(hits, key, f_list):
    lines = list(hits.get(key, []))
    for f in f_list:
        for a in f['addrs'] + f['old']:
            lines += hits.get(('addr', a), [])
    return sorted(set(lines), key=lambda x: (x.split(':')[0], int(x.split(':')[1])))


def write_reports(out, to, tn, bo, bn, findings, hits, status, new):
    g = group(findings)
    ob, nb = to['build'], tn['build']
    cnt = collections.Counter(status.values())
    head = ['# Packet diff %s -> %s' % (ob, nb), '',
            'c2s send calls: %d -> %d (ids %d -> %d); s2c registrations: %d -> %d (ids %d -> %d).' % (
                len(to['c2s']), len(tn['c2s']), len({c['id'] for c in to['c2s']}),
                len({c['id'] for c in tn['c2s']}), len(to['s2c']), len(tn['s2c']),
                len({e['id'] for e in to['s2c']}), len({e['id'] for e in tn['s2c']})),
            'Whole DLL (remap): %s; %d new functions.' % (
                ', '.join('%s %d' % (k, cnt[k]) for k in ('same', 'consts', 'refs', 'changed')), len(new)),
            'Gate %s, size %s, handler tables main %s / second %s (new build).' % (
                ' '.join('%08x' % a for a in tn['roles']['gate']), ' '.join('%08x' % a for a in tn['roles']['size']),
                ' '.join('%08x' % a for a in tn['roles']['reg']['main']),
                ' '.join('%08x' % a for a in tn['roles']['reg']['second'])), '']
    rep = list(head)
    summ = list(head)
    if not g:
        rep.append('No packet changes found.')
        summ.append('No packet changes found.')
    real = [(k, v) for k, v in g.items() if any(f['kind'] in REAL for f in v)]
    check = [(k, v) for k, v in g.items() if (k, v) not in real and any(f['kind'] in CHECK for f in v)]
    rest = [(k, v) for k, v in g.items() if (k, v) not in real and (k, v) not in check]
    for title, part in (('Changed packets', real),
                        ('Sender / handler code changed (read it: an offset move can read as a change)', check),
                        ('Packets whose sender / handler calls changed code, or whose registering function changed',
                         rest)):
        if not part:
            continue
        rep += ['## ' + title, '']
        summ += ['### ' + title, '', '| dir | id | change | new build | our code |', '|---|---|---|---|---|']
        for (dr, i), fl in part:
            files = our_files(hits, (dr, i), fl)
            unv = '' if all(f['ver'] for f in fl) else ' (check by hand: decompile and disassembly disagree)'
            rep.append('- %s 0x%03X%s' % (dr, i, unv))
            for f in fl:
                rep.append('  - %s: %s' % (f['kind'], f['detail']))
            rep.append('  - ours: %s' % (', '.join(files) if files else 'nothing names it'))
            maybe = sorted(set(hits.get(('maybe', dr, i), [])) - set(files))
            if maybe:
                rep.append('  - ours, direction not named on the line: %s' % ', '.join(maybe))
            kinds = sorted({f['kind'] for f in fl})
            where = sorted({'%08x' % a for f in fl for a in f['addrs']})
            fset = sorted({x.split(':')[0].replace('crates/vana-client/src/', '') for x in files})
            summ.append('| %s | 0x%03X | %s%s | %s | %s |' % (dr, i, ', '.join(kinds), '*' if unv else '',
                                                              ' '.join(where[:6]) + (' …' if len(where) > 6 else ''),
                                                              ', '.join('`%s`' % x for x in fset) or '-'))
        rep.append('')
        summ.append('')
    if any(not f['ver'] for f in findings):
        summ.append('\\* decompile and disassembly read this call differently; checked by hand before building.')
    unv = tn['unverified']
    if unv:
        rep += ['## Unverified call sites (new build)', '']
        for k, v in unv.items():
            rep.append('- %s %s: %s' % (k, bn.name(int(k, 16)), '; '.join(v)))
        rep.append('')
        summ.append('Call sites the two readings disagree on (new build): %d, listed in the local report.' % len(unv))
    rep += ['## Tables', '', 'Full tables: `%s/packets.tsv` and `%s/packets.tsv`.' % (bo.dir, bn.dir), '']
    summ += ['', 'Report only: nothing is changed or fixed by this job. The full report stays local.']
    with open(os.path.join(out, 'report.md'), 'w') as f:
        f.write('\n'.join(rep) + '\n')
    with open(os.path.join(out, 'summary.md'), 'w') as f:
        f.write('\n'.join(summ) + '\n')


def queue_rows(build, findings, hits):
    """(tier, kind, text) per packet: 'build' rows for confirmed changes, 'check' rows for changed code to read."""
    rows = []
    for (dr, i), fl in group(findings).items():
        kind = 'build' if any(f['kind'] in REAL for f in fl) else 'check' if any(
            f['kind'] in CHECK for f in fl) else None
        if not kind:
            continue
        files = our_files(hits, (dr, i), fl)
        what = '; '.join('%s: %s' % (f['kind'], f['detail']) for f in fl if f['kind'] in REAL + CHECK)
        todo = 'Read the code and build what changed' if kind == 'build' else \
            'Read the old and new code: build it if behaviour changed, else mark it checked'
        rows.append((kind, '%s 0x%03X (%s): %s. Ours: %s. %s (packet-diff report of %s).' % (
            dr, i, build, what, ', '.join(files[:8]) or 'nothing names it', todo, build)))
    return rows


def append_queue(path, build, rows):
    """Add the rows under the queue's last numbered row (once per build: the Source column carries the build)."""
    lines = open(path).read().split('\n')
    tag = 'packet diff %s' % build
    if not rows or any(tag in l for l in lines):
        return
    at = [k for k, l in enumerate(lines) if re.match(r'^\| \d+ \|', l)]
    n = max(int(lines[k].split('|')[1]) for k in at) if at else 0
    add = []
    for kind, r in rows:
        n += 1
        status = 'New build · Not built (%s)' % tag if kind == 'build' else 'New build · To check (%s)' % tag
        add.append('| %d | medium | 15 | %s | %s | %s |' % (n, tag, status, r.replace('|', '/')))
    k = at[-1] + 1 if at else len(lines)
    lines[k:k] = add
    open(path, 'w').write('\n'.join(lines))


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    sp = ap.add_subparsers(dest='cmd', required=True)
    t = sp.add_parser('table')
    t.add_argument('dir')
    d = sp.add_parser('diff')
    d.add_argument('old')
    d.add_argument('new')
    d.add_argument('--src', help='vana-client checkout to search (default: this one)')
    d.add_argument('--out')
    d.add_argument('--queue', help='modlog-gaps queue file to append the rows to')
    a = ap.parse_args()
    if a.cmd == 'table':
        tb, b = table(a.dir)
        print('%s: c2s %d calls (%d ids), s2c %d registrations (%d ids), unverified %d' % (
            tb['build'], len(tb['c2s']), len({c['id'] for c in tb['c2s']}), len(tb['s2c']),
            len({e['id'] for e in tb['s2c']}), len(tb['unverified'])))
    else:
        src = a.src or subprocess.run(['git', '-C', os.path.dirname(os.path.abspath(__file__)), 'rev-parse',
                                       '--show-toplevel'], capture_output=True, text=True).stdout.strip() or None
        out = a.out or os.path.join(a.new, 'packet-diff')
        f = diff(a.old, a.new, src, out, a.queue)
        print('%d findings on %d packets; report in %s' % (len(f), len(group(f)), out))


if __name__ == '__main__':
    main()
