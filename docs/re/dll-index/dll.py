"""Grep-only research on FFXiMain.unpacked.dll (no Ghidra/Rizin needed). Build the index once with ./ri.sh.

  dll.py addr ADDR          everything: function C, callers, callees, strings used, notes mentions
  dll.py fn ADDR...         pseudo-C of the function containing each address
  dll.py callers ADDR       who references the function containing ADDR (--exact: just ADDR)
  dll.py callees ADDR       calls/refs made from the function containing ADDR
  dll.py str REGEX          strings matching REGEX and the functions using them
  dll.py grep REGEX         functions whose pseudo-C matches REGEX (address + matching lines)
  dll.py notes ADDR         where ADDR (or its function) is mentioned in our notes
  dll.py build-notes [PATH..] (re)build notes_index.tsv from .md/.rs/.lua files/dirs (default: repo MODLOG.md + docs/re)
  dll.py names REGEX        list function names (names.tsv + names_manual.tsv) matching REGEX
  dll.py name ADDR NAME     give the function at ADDR a hand-checked name (appends to names_manual.tsv)
ADDR may be hex (0x.., FUN_..) or a function name. Output shows names next to addresses and in the C.
Generated files are read from $FFXI_RE_DIR (default: ./work next to this script); names*.tsv live next to this script.
"""
import sys, os, re, bisect, collections
H = os.path.dirname(os.path.abspath(__file__))
D = os.path.abspath(os.environ.get('FFXI_RE_DIR') or os.path.join(H, 'work')) + '/'
REPO = os.path.abspath(os.path.join(H, '..', '..', '..'))
def P(n):
    return os.path.join(H, n) if n.startswith('names') else D + n
def need(n):
    if not os.path.exists(P(n)): sys.exit('missing %s -- run ./ri.sh first (work dir: %s)' % (P(n), D))
    return P(n)

_N = None
def names():
    """addr -> (name, conf, source); names_manual.tsv wins over names.tsv."""
    global _N
    if _N is None:
        _N = {}
        for fn_, man in (('names.tsv', False), ('names_manual.tsv', True)):
            if not os.path.exists(P(fn_)): continue
            for l in open(P(fn_)):
                if l.startswith('#') or not l.strip(): continue
                f = l.rstrip('\n').split('\t')
                if len(f) < 2: continue
                _N[int(f[0].replace('0x', ''), 16)] = (f[1], 'high' if man else f[2], 'manual' if man else f[3].replace('+new', ''))
    return _N
_R = None
def byname(n):
    global _R
    if _R is None: _R = {v[0]: k for k, v in names().items()}
    return _R.get(n)
def nm(a):
    v = names().get(a if isinstance(a, int) else int(a, 16)) if a not in ('-', None) else None
    return v[0] if v else None
def lab(a):
    """'1012ac60 win_common_onkey' for named addresses, else the address."""
    if a in ('-', None): return '-'
    s = a if isinstance(a, str) else '%08x' % a
    n = nm(s); return s + ' ' + n if n else s
_FUNRX = re.compile(r'\bFUN_(10[0-9a-f]{6})\b')
def named_c(c):
    return _FUNRX.sub(lambda m: nm(m.group(1)) or m.group(0), c)
def h(a):
    a = a.replace('0x', '')
    b = byname(a)
    if b is not None: return b
    try: return int(a.replace('FUN_', '').replace('thunk_', ''), 16)
    except ValueError: sys.exit('unknown name or address: %s  (try: dll.py names REGEX)' % a)

_F = None
def funcs():
    global _F
    if _F is None:
        rows = []
        if os.path.exists(P('funcs_full.tsv')):
            for l in open(P('funcs_full.tsv')):
                e, end, name, src = l.rstrip('\n').split('\t')
                rows.append((int(e, 16), int(end, 16), name))
        rows.sort(); _F = rows
    return _F
def owner(a):
    F = funcs(); i = bisect.bisect_right(F, (a, 1 << 40, '')) - 1
    if i >= 0 and F[i][0] <= a <= F[i][1]: return F[i][0]
    return None

_C = None
def decomp():
    global _C
    if _C is None:
        f = P('all_decomp_full.c') if os.path.exists(P('all_decomp_full.c')) else P('all_decomp.c')
        parts = re.split(r'^//@@FUNC ([0-9a-f]{8})\n', open(f if os.path.exists(f) else need('all_decomp_full.c'), errors='replace').read(), flags=re.M)
        _C = {int(parts[i], 16): parts[i + 1] for i in range(1, len(parts), 2)}
    return _C
def fn(a):
    C = decomp(); k = owner(a)
    if k is None:
        ks = sorted(C); k = ks[bisect.bisect_right(ks, a) - 1]
    return k, C.get(k, '// not decompiled\n')

def xrows():
    for l in open(need('xrefs.tsv')):
        yield l.rstrip('\n').split('\t')
def callers(a, exact=False):
    t = '%08x' % (a if exact else (owner(a) or a))
    return [r for r in xrows() if (r[2] == t if exact else r[3] == t and r[1] != t)]
def callees(a):
    t = '%08x' % (owner(a) or a)
    return [r for r in xrows() if r[1] == t and r[3] != t]
def strs(rx=None, func=None):
    out = []
    for l in open(need('strrefs.tsv'), encoding='utf-8', errors='replace'):
        s, frm, f, txt = l.rstrip('\n').split('\t', 3)
        if func and f != func: continue
        if rx and not re.search(rx, txt): continue
        out.append((s, frm, f, txt))
    return out
def notes(a):
    if not os.path.exists(P('notes_index.tsv')): return []
    keys = {'%08x' % a}
    k = owner(a)
    if k: keys.add('%08x' % k)
    return [l.rstrip('\n').split('\t', 2) for l in open(P('notes_index.tsv'), errors='replace') if l.split('\t', 1)[0] in keys]

def show_x(rows, col):
    by = collections.OrderedDict()
    for r in rows: by.setdefault(r[col], []).append(r)
    for f, rs in by.items(): print('  %s  (%d: %s)' % (lab(f), len(rs), ', '.join(sorted({r[4] for r in rs}))))

def main(a):
    if not a or a[0] in ('-h', '--help'): print(__doc__); return
    cmd, args = a[0], a[1:]
    if cmd == 'fn':
        for x in args:
            k, c = fn(h(x)); print('//==== %s (for %s)' % (lab(k), x)); print(named_c(c))
    elif cmd == 'callers':
        show_x(callers(h(args[0]), '--exact' in args), 1)
    elif cmd == 'callees':
        show_x(callees(h(args[0])), 3)
    elif cmd == 'str':
        for s, frm, f, txt in strs(args[0]): print('%s  %s in %s  %r' % (s, frm, lab(f), txt))
    elif cmd == 'grep':
        rx = re.compile(args[0])
        for k, c in sorted(decomp().items()):
            c = named_c(c)
            hits = [l.strip() for l in c.splitlines() if rx.search(l)]
            if hits: print('%s: ' % lab(k) + ' | '.join(hits[:3]) + (' ...' if len(hits) > 3 else ''))
    elif cmd == 'notes':
        for addr, where, line in notes(h(args[0])): print('%s  %s  %s' % (lab(addr), where, line))
    elif cmd == 'addr':
        x = h(args[0]); k, c = fn(x)
        print('//==== %s (for %08x)' % (lab(k), x)); print(named_c(c))
        print('== callers'); show_x(callers(x), 1)
        print('== callees'); show_x(callees(x), 3)
        print('== strings'); [print('  %s %r' % (s, t)) for s, _, _, t in strs(func='%08x' % k)]
        print('== notes'); [print('  %s  %s' % (w, l)) for _, w, l in notes(x)]
    elif cmd == 'build-notes':
        rx = re.compile(r'(?:FUN_|LAB_|DAT_|0x)?(10[0-9a-fA-F]{6})\b')
        n = 0
        roots = args or [os.path.join(REPO, 'MODLOG.md'), os.path.join(REPO, 'docs', 're')]
        os.makedirs(D, exist_ok=True)
        with open(P('notes_index.tsv'), 'w') as w:
            for root in roots:
                walk = os.walk(root) if os.path.isdir(root) else [(os.path.dirname(root), [], [os.path.basename(root)])]
                for dp, dn, fns in walk:
                    dn[:] = [d for d in dn if d not in ('.git', 'target', 'vendor', 'node_modules')]
                    for f in fns:
                        if not f.endswith(('.md', '.rs', '.lua')): continue
                        p = os.path.join(dp, f)
                        try:
                            for i, l in enumerate(open(p, errors='replace'), 1):
                                for m in set(rx.findall(l)):
                                    w.write('%s\t%s:%d\t%s\n' % (m.lower(), p, i, l.strip()[:300])); n += 1
                        except OSError: pass
        print('notes_index.tsv:', n, 'mentions')
    elif cmd == 'names':
        rx = re.compile(args[0] if args else '.')
        for a, (n, c, src) in sorted(names().items()):
            if rx.search(n): print('%08x  %-40s %-4s %s' % (a, n, c, src))
    elif cmd == 'name':
        a, n = h(args[0]), args[1]
        if not re.match(r'[A-Za-z_][A-Za-z0-9_]*$', n): sys.exit('name must be a C identifier')
        k = owner(a)
        if k is not None and k != a: print('note: %08x is inside %s; naming the function entry' % (a, lab(k))); a = k
        o = byname(n)
        if o is not None and o != a: sys.exit('%s is already %08x' % (n, o))
        new = not os.path.exists(P('names_manual.tsv'))
        with open(P('names_manual.tsv'), 'a') as w:
            if new: w.write('# addr<TAB>name[<TAB>note]  -- hand-checked names; always win over build_names.py.\n')
            w.write('%08x\t%s\t%s\n' % (a, n, ' '.join(args[2:])))
        print('%08x -> %s (names_manual.tsv)' % (a, n))
    else: print(__doc__)

if __name__ == '__main__': main(sys.argv[1:])
