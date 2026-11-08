"""Build names.tsv (addr  name  confidence  source) for FFXiMain.unpacked.dll function entries.

Sources, highest priority first (a later source never overrides an earlier one):
  manual   names_manual.tsv (always wins)
  ghidra   real symbols Ghidra already had (exports, named imports)
  evtable  event-op handlers from the event dispatcher's jump table in the DLL -> evop_XX (lowest op)
  onkey    window OnKey handlers (menu_classes/onkey_scan/event_handlers/winclasses) -> win_<menu>_onkey
  keyfn    key-function callbacks (keyfn_table.tsv)        -> keyfn_XX_cb
  vtable   window class virtual methods (menu_classes vtables, read from the DLL) -> win_<menu>_vOFF
  notes    "FUN_x (desc)", "FUN_x = desc", "FUN_x: desc" in notes_index.tsv
  imp      thunks that only jump to an import              -> imp_<dll>_<func>
  menu     functions using exactly one "menu    xxxx" string -> menu_<xxxx>_open / _fn
  srcfile  functions citing exactly one C:\\dev\\...\\x.c path -> src_<x>
  string   functions whose only string is a log/format string -> str_<words>
  thunk    Ghidra thunk_FUN_x whose target got a name     -> thunk_<target name>
Run: python3 build_names.py   (writes names.tsv next to this script, prints counts)

Inputs are read from $FFXI_RE_DIR (default: ./work next to this script). Only funcs_full.tsv (from ./ri.sh) is required.
Every other input is optional; when a source's input is missing, that source's rows are carried over unchanged from
the current names.tsv, so a partial setup never loses names:
  FFXiMain.unpacked.dll ($FFXI_DLL)       evtable, vtable
  menu_classes.json, onkey_scan.json, event_handlers.json, winclasses.json   onkey (+ vtable: menu_classes.json)
  keyfn_table.tsv                         keyfn
  notes_index.tsv  (dll.py build-notes)   notes
  imports.txt      (rizin -q -c ii DLL)   imp
  strrefs.tsv, xrefs.tsv (./ri.sh)        menu, srcfile, string, imp
  funcs_full.orig.tsv (ri_named.sh keeps it)  ghidra (falls back to funcs_full.tsv)
"""
import os, re, sys, json, struct, collections
H = os.path.dirname(os.path.abspath(__file__))
D = os.path.abspath(os.environ.get('FFXI_RE_DIR') or os.path.join(H, 'work')) + '/'
DLL = os.environ.get('FFXI_DLL') or D + 'FFXiMain.unpacked.dll'
def P(n): return os.path.join(H, n) if n.startswith('names') else D + n
def have(*ns): return all(os.path.exists(P(n)) for n in ns)
if not have('funcs_full.tsv'): sys.exit('missing %s -- run ./ri.sh first' % P('funcs_full.tsv'))

# the current names.tsv: rows of sources whose inputs are missing are carried over from it
OLD = collections.defaultdict(list)        # source -> [(addr, name, conf, label)]
if os.path.exists(P('names.tsv')):
    for l in open(P('names.tsv')):
        if l.startswith('#') or not l.strip(): continue
        f = l.rstrip('\n').split('\t')
        if len(f) >= 4: OLD[f[3].split()[0].replace('+new', '')].append((int(f[0], 16), f[1], f[2], f[3]))
skipped = []

ENT = {}                                   # entry -> (end, ghidra name)
for l in open(P('funcs_full.tsv')):
    e, end, nm, src = l.rstrip('\n').split('\t')
    ENT[int(e, 16)] = (int(end, 16), nm)

STOP = {'the', 'a', 'an', 'of', 'to', 'and', 'or', 'is', 'its', 'it', 'this', 'that', 'for', 'in', 'on', 'at', 'by', 'with', 'from'}
BAD_START = STOP | {'if', 'when', 'which', 'was', 'not', 'no', 'only', 'called', 'via', 'then', 'also', 'see', 'eg', 'ie', 'case',
    'same', 'returns', 'return', 'asm', 'c', 'id', 'arg', 'args', 'param', 'mode', 'kind', 'type', 'op', 'flag', 'bit', 'byte',
    'field', 'slot', 'value', 'val', 'var', 'u8', 'u16', 'u32', 'i8', 'i16', 'i32', 'f32', 'dat', 'fun', 'lab', 'ptr', 'old',
    'new', 'both', 'all', 'every', 'each', 'one', 'two', 'as', 'but', 'so', 'not', 'nothing', 'none', 'yes', 'true', 'false',
    'here', 'there', 'they', 'we', 'our', 'i', 'line', 'lines', 'build', 'retail', 'older', 'newer', 'up', 'down', 'still',
    'unused', 'unknown', 'maybe', 'probably', 'likely', 'e', 'etc', 'about', 'after', 'before', 'until', 'while', 'where', 'how',
    'always', 'never', 'full', 'signed', 'unsigned', 'instances', 'drawn', 'applied', 'stored', 'set', 'sets', 'read',
    'reads', 'written', 'cleared', 'used', 'uses', 'other', 'more', 'less', 'any', 'some', 'again', 'just', 'now', 'even',
    'vtable', 'flags', 'cases', 'entity', 'index', 'table', 'what', 'who', 'his', 'her', 'their', 'has', 'have', 'had', 'be', 'been', 'are', 'were', 'does', 'did', 'do', 'can', 'may'}

def snake(s, maxlen=40):
    s = re.sub(r'%[-+ #0-9.]*[a-zA-Z]+', ' ', s)            # printf specs
    s = re.sub(r'\\[nrt]', ' ', s)
    s = re.sub(r'[^A-Za-z0-9]+', '_', s).strip('_').lower()
    s = re.sub(r'_+', '_', s)
    if len(s) > maxlen: s = s[:maxlen].rsplit('_', 1)[0]
    if s and s[0].isdigit(): s = 'n' + s
    return s

names = {}                                 # addr -> (name, conf, source)
used = set()
counts = collections.Counter()
import bisect
_E = sorted(ENT)
def inside(a):
    i = bisect.bisect_right(_E, a) - 1
    while i >= 0 and ENT[_E[i]][0] - _E[i] > 0x10000: i -= 1   # skip bogus huge rizin spans
    return i >= 0 and _E[i] <= a <= ENT[_E[i]][0]
NEW = set()                                # code addresses (vtable/OnKey targets) Ghidra has no function for yet
def put(a, nm, conf, src, raw=False, label=None, force=False):
    if a in NEW or force: pass
    elif a not in ENT or a in names or not nm: return False
    if a in names or not nm: return False
    nm = re.sub(r'\W', '_', nm) if raw else snake(nm)
    if not nm or not re.match(r'[A-Za-z_][A-Za-z0-9_]*$', nm): return False
    base, k = nm, 2
    while nm in used: nm = '%s_%d' % (base, k); k += 1
    used.add(nm); names[a] = (nm, conf, label or src); counts[(src, conf)] += 1
    return True
def carry(src, why):
    """input for src is missing: keep its rows from the current names.tsv"""
    skipped.append('%s (no %s; %d rows kept)' % (src, why, len(OLD[src])))
    for a, nm, conf, label in OLD[src]:
        if label.endswith('+new'): NEW.add(a); label = label[:-4]
        put(a, nm, conf, src, raw=True, label=label, force=True)
def hx(s): return int(str(s).replace('0x', ''), 16) if isinstance(s, str) else int(s)

# --- manual -----------------------------------------------------------------
if not os.path.exists(P('names_manual.tsv')):
    open(P('names_manual.tsv'), 'w').write('# addr<TAB>name[<TAB>note]  -- hand-checked names; always win over build_names.py.\n'
                                           '# Add with: python3 dll.py name ADDR NAME\n')
for l in open(P('names_manual.tsv')):
    if l.startswith('#') or not l.strip(): continue
    f = l.rstrip('\n').split('\t')
    if len(f) >= 2:
        a = hx(f[0])
        names.pop(a, None)
        put(a, f[1], 'high', 'manual', raw=True)

# --- ghidra's own real names (exports / IAT-named thunks) -------------------
# funcs_full.orig.tsv = the export before any names were applied (ri_named.sh keeps it)
ORIG = {}
for l in open(P('funcs_full.orig.tsv') if os.path.exists(P('funcs_full.orig.tsv')) else P('funcs_full.tsv')):
    f = l.rstrip('\n').split('\t'); ORIG[int(f[0], 16)] = f[2]
for a, nm in ORIG.items():
    if not re.match(r'(FUN_|thunk_|LAB_|SUB_)', nm): put(a, nm, 'high', 'ghidra', raw=True)   # keep Ghidra's spelling

# --- the DLL (evtable, vtable) ---------------------------------------------------
dll = open(DLL, 'rb').read() if os.path.exists(DLL) else None
secs = []; TEXT = []
if dll:
    pe = struct.unpack_from('<I', dll, 0x3c)[0]
    nsec = struct.unpack_from('<H', dll, pe + 6)[0]; opt = struct.unpack_from('<H', dll, pe + 20)[0]
    ib = struct.unpack_from('<I', dll, pe + 24 + 28)[0]
    for i in range(nsec):
        o = pe + 24 + opt + 40 * i
        vs, va, rs, ro = struct.unpack_from('<IIII', dll, o + 8)
        ch = struct.unpack_from('<I', dll, o + 36)[0]
        secs.append((ib + va, ib + va + max(vs, rs), ro, rs)); TEXT.append((ib + va, ib + va + vs, ch & 0x20000000))
def rd(va, n=4):
    for s, e, ro, rs in secs:
        if s <= va < e and va - s + n <= rs: return int.from_bytes(dll[ro + va - s:ro + va - s + n], 'little')
    return None

# --- event ops: the event script dispatcher (FUN_100bc280 in build 30261006_0) ------
#   movsx eax, word [esp+4]; cmp eax, 0xda; ja default; jmp [eax*4 + 0x100bc960]
# every case is "call handler; ret 4". vana-client's event_ops.rs was decoded against an older DLL build, so its
# FUN_ addresses do not match this one; the table here is the source of truth for this build.
EVT_DISPATCH, EVT_TABLE, EVT_MAX = 0x100bc280, 0x100bc960, 0xda
def evtable():
    if rd(EVT_DISPATCH, 4) != 0x2444bf0f or rd(EVT_DISPATCH + 5, 1) != 0x3d or rd(EVT_DISPATCH + 6) != EVT_MAX \
            or rd(EVT_DISPATCH + 0x10, 3) != 0x8524ff or rd(EVT_DISPATCH + 0x13) != EVT_TABLE:
        return None                                  # another DLL build: the dispatcher moved
    ops = collections.defaultdict(list)
    for op in range(EVT_MAX + 1):
        lab = rd(EVT_TABLE + 4 * op)
        if lab is None or rd(lab, 1) != 0xe8: return None
        ops[(lab + 5 + struct.unpack('<i', rd(lab + 1).to_bytes(4, 'little'))[0]) & 0xffffffff].append(op)
    return ops
ev = evtable() if dll else None
if ev is None: carry('evtable', 'DLL' if not dll else 'matching event dispatcher in the DLL')
else:
    for a, ops in sorted(ev.items(), key=lambda x: x[1][0]):
        label = 'evtable' if len(ops) == 1 else 'evtable ops ' + ','.join('0x%02x' % o for o in ops)
        put(a, 'evop_%02x' % ops[0], 'high', 'evtable', raw=True, label=label)

# --- window OnKey handlers -----------------------------------------------------
def mname(s): return s.replace('menu', '', 1).strip() if s.startswith('menu ') else s.strip()
onkey = collections.defaultdict(set)
mc = json.load(open(P('menu_classes.json'))) if have('menu_classes.json') else {}
for k, v in mc.items():
    for o in v['onkey']: onkey[hx(o)].add(mname(k))
if have('onkey_scan.json'):
    for o, v in json.load(open(P('onkey_scan.json'))).items(): onkey[hx(o)].update(v['names'])
if have('event_handlers.json'):
    for ev_id, lst in json.load(open(P('event_handlers.json'))).items():
        for o, ms in lst: onkey[hx(o)].update(ms)
if have('winclasses.json'):
    for w in json.load(open(P('winclasses.json'))):
        onkey[w['onkey']].update(w['names'])
def textcode(a):
    return a is not None and any(s <= a < e and x for s, e, x in TEXT) and not inside(a) and a not in ENT
for a in onkey:
    if textcode(a): NEW.add(a)
per_menu = collections.Counter(m for ms in onkey.values() if len(ms) == 1 for m in ms)
for a, ms in sorted(onkey.items(), key=lambda x: (len(x[1]) or 99, x[0])):
    ms = sorted(ms)
    if len(ms) == 1: put(a, 'win_%s_onkey' % ms[0], 'high' if per_menu[ms[0]] == 1 else 'med', 'onkey')   # several handlers for one menu: ambiguous
    elif 2 <= len(ms) <= 4: put(a, 'win_%s_onkey' % ms[0], 'med', 'onkey')
    elif len(ms) > 4: put(a, 'win_common_onkey', 'low', 'onkey')     # shared base handler
ONKEY_IN = ('menu_classes.json', 'onkey_scan.json', 'event_handlers.json', 'winclasses.json')
if not all(have(n) for n in ONKEY_IN):              # some inputs missing: keep their old rows (derived names above win)
    carry('onkey', ', '.join(n for n in ONKEY_IN if not have(n)))

# --- key-function callbacks ---------------------------------------------------
kf = collections.defaultdict(list)
if not have('keyfn_table.tsv'): carry('keyfn', 'keyfn_table.tsv')
for l in list(open(P('keyfn_table.tsv')))[1:] if have('keyfn_table.tsv') else []:
    f = l.rstrip('\n').split('\t')
    if len(f) > 3 and re.match(r'[0-9a-f]{8}$', f[3]): kf[int(f[3], 16)].append(int(f[0], 16))
for a, ids in sorted(kf.items()):
    put(a, 'keyfn_' + '_'.join('%02x' % i for i in ids[:3]) + '_cb', 'high' if len(ids) <= 2 else 'med', 'keyfn')

slot_menus = collections.defaultdict(lambda: collections.defaultdict(set))   # func -> vt -> {(menu, off)}
vt_menus = collections.defaultdict(set)
if not (mc and dll): carry('vtable', 'menu_classes.json + DLL'); mc = {}
for k, v in mc.items():
    for vt in v['vtables']: vt_menus[hx(vt)].add(mname(k))
for vt, ms in vt_menus.items():
    for i in range(64):
        f = rd(vt + 4 * i)
        if i and vt + 4 * i in vt_menus: break     # ran into the next vtable
        if textcode(f): NEW.add(f)
        elif f not in ENT: break
        for m in ms: slot_menus[f][vt].add((m, 4 * i))
for a, vts in sorted(slot_menus.items()):
    if len(vts) != 1: continue                       # shared by several classes: base-class method
    (vt, ms), = vts.items()
    menus = sorted({m for m, _ in ms}); off = min(o for _, o in ms)
    meth = 'onkey' if off == 0x18 else 'v%02x' % off
    put(a, 'win_%s_%s' % (menus[0], meth), 'med' if len(menus) == 1 else 'low', 'vtable')

# --- notes -------------------------------------------------------------------
A = r'(?:FUN_|0x)(10[0-9a-f]{6})\b\**`?\**'
pats = [re.compile(A + r'\s+\(([^()]{3,60})\)'),                                       # FUN_x (desc)
        re.compile(A + r'\s*(?:=|—|–| - )\s*\**([A-Za-z][^.;,()|:"`*=]{2,60})'),      # FUN_x = desc
        re.compile(r'^[\s/#*|>\-]*' + A + r'\s*[:|]\s*\**([A-Za-z][^.;,()|:"`*=]{2,60})')]  # FUN_x: desc / | FUN_x | desc
votes = collections.defaultdict(collections.Counter)
seen = set()
def good_desc(d):
    d = re.sub(r'`', '', d).strip()
    w = re.findall(r'[A-Za-z][A-Za-z0-9]*', d)
    if not 1 <= len(w) <= 5 or len(re.findall(r'[^\w\s\-/\'.]', d)) > 1: return None
    if w[0].lower() in BAD_START or re.match(r'(?:0x|\d|\+|DAT_|FUN_|param)', d): return None
    if not any(len(x) >= 4 for x in w): return None
    if any(re.match(r'(?:modlog|inf|inferred|confirmed|unconfirmed|fun|dat|lab|l\d+|[a-f0-9]*\d[a-f0-9]*[a-f][a-f0-9]*)$', x.lower()) for x in w): return None
    while w and w[0].lower() in STOP: w = w[1:]
    return '_'.join(w) if w else None
if not have('notes_index.tsv'): carry('notes', 'notes_index.tsv; run dll.py build-notes')
for l in open(P('notes_index.tsv'), errors='replace') if have('notes_index.tsv') else []:
    f = l.rstrip('\n').split('\t', 2)
    if len(f) < 3 or 'Some(Op' in f[2]: continue
    path, txt = f[1].rsplit(':', 1)[0], f[2]
    key = (os.path.basename(path), txt)
    if key in seen: continue                       # same line in several worktrees
    seen.add(key)
    # re-notes and notes kept in the work dir are about this build; MODLOG and the vana-client repo (docs/re, sources,
    # re-notes/*-wt checkouts) are often about an older DLL, so they need two agreeing lines before they count
    this_build = ('/re-notes/' in path and '-wt/' not in path) or (path.startswith(D) and '/vana-client/' not in path)
    w = 2.0 if this_build else 0.5
    for p in pats:
        for m in p.finditer(txt):
            a = int(m.group(1), 16)
            if a not in ENT: continue                     # interior / other-build addresses
            d = good_desc(m.group(2))
            if d: votes[a][snake(d)] += w
for a, c in sorted(votes.items()):
    (nm, top), tot = c.most_common(1)[0], sum(c.values())
    conf = 'med' if top >= 3 and top / tot >= 0.6 else 'low'
    if top < 1.0 or top / tot < 0.4: continue                          # notes disagree too much
    if '_' not in nm and top < 3: continue
    if re.search(r'(?:^|_)x[0-9a-f]{2,}', nm) and not re.match(r'(?:s2c|c2s)_', nm): continue   # offsets / constants, not names                # one word from one line: too vague
    put(a, nm, conf, 'notes')

# --- import thunks -------------------------------------------------------------
imp = {}
if not have('imports.txt', 'xrefs.tsv'): carry('imp', 'imports.txt (rizin -q -c ii DLL > imports.txt)')
for l in open(P('imports.txt')) if have('imports.txt', 'xrefs.tsv') else []:
    m = re.match(r'\s*\d+\s+0x([0-9a-f]+)\s+\S+\s+FUNC\s+(\S+)\s+(\S+)', l)
    if m: imp[int(m.group(1), 16)] = (m.group(2).rsplit('.', 1)[0], m.group(3))
xout = collections.defaultdict(list)
for l in open(P('xrefs.tsv')) if have('xrefs.tsv') else []:
    fr, ff, to, tf, ty = l.rstrip('\n').split('\t')
    if ff != '-': xout[int(ff, 16)].append((int(to, 16), ty))
for a, (end, nm) in ENT.items() if imp else ():
    if end - a > 8: continue
    t = [to for to, ty in xout.get(a, []) if to in imp]
    if len(t) == 1 and len(xout[a]) <= 2:
        put(a, 'imp_%s_%s' % imp[t[0]], 'high', 'imp')

# --- strings -------------------------------------------------------------------
fstr = collections.defaultdict(set)
if not have('strrefs.tsv', 'xrefs.tsv'):
    for src in ('menu', 'srcfile', 'string'): carry(src, 'strrefs.tsv/xrefs.tsv')
for l in open(P('strrefs.tsv'), encoding='utf-8', errors='replace') if have('strrefs.tsv', 'xrefs.tsv') else []:
    s, frm, fn, txt = l.rstrip('\n').split('\t', 3)
    if fn != '-': fstr[int(fn, 16)].add(txt)
decomp_open = {a for a, outs in xout.items() if any(to == 0x1015e350 for to, ty in outs)}   # calls open-window-by-name
for a, ss in sorted(fstr.items()):
    menus = {s[4:].strip() for s in ss if s.startswith('menu    ')}
    if len(menus) == 1:
        put(a, 'menu_%s_%s' % (menus.pop(), 'open' if a in decomp_open else 'fn'), 'med' if a in decomp_open else 'low', 'menu'); continue
    paths = {re.search(r'\\([A-Za-z0-9_]+)\.(?:c|cpp)$', s).group(1) for s in ss if re.search(r'\\[A-Za-z0-9_]+\.(?:c|cpp)$', s)}
    if len(paths) == 1:
        put(a, 'src_' + paths.pop(), 'low', 'srcfile'); continue
    real = [s for s in ss if len(re.findall(r'[A-Za-z]{3,}', s)) >= 2]
    if len(real) == 1 and len(ss) <= 2:
        s = real[0]
        if re.search(r'%|\\n|[a-z] [a-z]', s) and not re.search(r'[\\/:]\w+\.\w+$', s):
            put(a, 'str_' + snake(s, 36), 'low', 'string')

# --- thunks to named functions ----------------------------------------------------
for _ in range(3):
    for a, nm in ORIG.items():
        m = re.match(r'thunk_FUN_(10[0-9a-f]{6})$', nm)
        if m and int(m.group(1), 16) in names and a not in names:
            tn, tc, _s = names[int(m.group(1), 16)]
            put(a, 'thunk_' + tn, tc, 'thunk')

# --- names at addresses this export does not know (e.g. ri.sh ran without rizin's funcs.txt): keep them -------
kept = 0
for src, rows in OLD.items():
    for a, nm, conf, label in rows:
        if a in ENT or a in names or src == 'manual': continue
        if label.endswith('+new'): NEW.add(a); label = label[:-4]
        kept += put(a, nm, conf, src, raw=True, label=label, force=True)
if kept: skipped.append('%d names at addresses missing from funcs_full.tsv' % kept)

with open(P('names.tsv'), 'w') as w:
    w.write('# addr\tname\tconfidence\tsource   (generated by build_names.py; edit names_manual.tsv instead)\n')
    for a in sorted(names): w.write('%08x\t%s\t%s\t%s%s\n' % ((a,) + names[a] + ('+new' if a in NEW else '',)))
print('names.tsv: %d names (%d at code Ghidra had no function for; ApplyNames creates them)' % (len(names), len(NEW & set(names))))
for s in sorted({s for s, _ in counts}, key=lambda s: -sum(v for (x, _), v in counts.items() if x == s)):
    print('  %-8s ' % s + '  '.join('%s %d' % (c, counts[(s, c)]) for c in ('high', 'med', 'low') if counts[(s, c)]))
for x in skipped: print('  kept from the previous names.tsv: ' + x)
