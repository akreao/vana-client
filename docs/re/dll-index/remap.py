#!/usr/bin/env python3
"""Match the functions and data of two FFXiMain.dll builds, and list what changed between them.

    python3 remap.py OLD_DIR NEW_DIR [--out DIR]

Each dir is a dll-index work dir holding FFXiMain.unpacked.dll and sig_insns.tsv (ri.sh writes it: ExportSig.java,
after FixVt.java). Output goes to NEW_DIR/remap/ (generated from the DLLs: never commit it):

    funcmap.tsv    old entry, new entry, status, similarity, instructions old / new, old name
                   status: same     identical code, every address it uses maps to the matching place
                           refs     identical code, but an address it uses points somewhere else now
                           consts   the same instructions, only constants differ (a struct grew: offsets shifted);
                                    the last column lists the shifts
                           changed  the code differs (see changed/<old>_<new>.diff)
    new.tsv        functions only in the new build (entry, last address, instructions, callers in the new build)
    removed.tsv    functions only in the old build
    datamap.tsv    old data address -> new data address, with the number of matched instructions that agree
    conflicts.tsv  addresses that matched instructions map to more than one place
    data_diff.tsv  initialised data (.rdata / .data, and tables inside .text) whose bytes differ once moved
    changed/       one instruction diff per changed / refs function
    summary.txt    the counts

How it matches. ExportSig blanks what moves between builds (the bytes under each base relocation, and the
displacement of relative calls / jumps that leave the function), so an untouched function has the same blanked bytes
in both builds. The two function lists are aligned in address order on those bytes (the linker keeps the order);
what is left between aligned runs is paired by instruction similarity. Every blanked address of a matched instruction
then votes for where its target went; the votes give the data map and check the code map.
"""
import sys, os, struct, bisect, difflib, hashlib, collections, argparse


class Pe:
    def __init__(self, path):
        self.d = d = open(path, 'rb').read()
        nt = struct.unpack_from('<I', d, 0x3c)[0]
        nsec, optsz = struct.unpack_from('<H', d, nt + 6)[0], struct.unpack_from('<H', d, nt + 20)[0]
        self.base = struct.unpack_from('<I', d, nt + 24 + 28)[0]
        self.secs = []
        for i in range(nsec):
            s = nt + 24 + optsz + i * 40
            name = d[s:s + 8].rstrip(b'\0').decode()
            vsize, va, rsize, raw = struct.unpack_from('<IIII', d, s + 8)
            chars = struct.unpack_from('<I', d, s + 36)[0]
            self.secs.append((name, self.base + va, vsize, raw, rsize, chars))
        self.relocs = self._relocs()
        self.relset = set(self.relocs)

    def sec(self, va):
        for s in self.secs:
            if s[1] <= va < s[1] + max(s[2], s[4]):
                return s
        return None

    def off(self, va):
        s = self.sec(va)
        if s and va - s[1] < s[4]:
            return s[3] + va - s[1]
        return None

    def bytes(self, va, n):
        """Up to `n` initialised bytes at `va` (never past its section's file data); None outside."""
        s = self.sec(va)
        if s is None or va - s[1] >= s[4]:
            return None
        o = s[3] + va - s[1]
        return self.d[o:o + min(n, s[4] - (va - s[1]))]

    def imports(self):
        """IAT slot -> 'dll!name', and the address ranges of the import tables (they hold RVAs, not pointers)."""
        nt = struct.unpack_from('<I', self.d, 0x3c)[0]
        rva, size = struct.unpack_from('<II', self.d, nt + 24 + 96 + 8)
        names, ranges = {}, [(self.base + rva, self.base + rva + size)]
        o = self.off(self.base + rva)
        lo, hi = None, None
        while o is not None:
            oft, _, _, nm, ft = struct.unpack_from('<IIIII', self.d, o)
            if not nm and not ft:
                break
            dll = self.cstr(self.base + nm)
            k = 0
            while True:
                v = self.u32(self.base + (oft or ft) + k * 4)
                if not v:
                    break
                names[self.base + ft + k * 4] = dll.lower() + '!' + ('#%d' % (v & 0xffff) if v >> 31 else self.cstr(self.base + v + 2))
                if not v >> 31:
                    lo, hi = min(lo or v, v), max(hi or v, v)
                k += 1
            ranges.append((self.base + ft, self.base + ft + k * 4 + 4))
            if oft:
                ranges.append((self.base + oft, self.base + oft + k * 4 + 4))
            lo, hi = min(lo or nm, nm), max(hi or nm, nm)
            o += 20
        if lo:
            ranges.append((self.base + lo, self.base + hi + 0x100))
        return names, ranges

    def cstr(self, va):
        o = self.off(va)
        return self.d[o:self.d.index(b'\0', o)].decode('latin-1') if o is not None else ''

    def u32(self, va):
        b = self.bytes(va, 4)
        return struct.unpack('<I', b)[0] if b and len(b) == 4 else None

    def _relocs(self):
        s = next(s for s in self.secs if s[0] == '.reloc')
        b = self.d[s[3]:s[3] + s[4]]
        out, o = [], 0
        while o + 8 <= len(b):
            page, size = struct.unpack_from('<II', b, o)
            if size < 8:
                break
            for k in range(o + 8, min(o + size, len(b) - 1), 2):
                e = struct.unpack_from('<H', b, k)[0]
                if e >> 12 == 3:
                    out.append(self.base + page + (e & 0xfff))
            o += size
        return sorted(out)

    def is_code(self, va):
        s = self.sec(va)
        return bool(s and s[0] == '.text')


class Func:
    __slots__ = ('entry', 'end', 'name', 'ins', 'key', 'lim')


def branch_len(hx):
    """Bytes of relative displacement at the end of a branch / call instruction (0 when it is none)."""
    op, n = int(hx[:2], 16), len(hx) // 2
    if op in (0xe8, 0xe9) and n == 5:
        return 4
    if op == 0x0f and n == 6 and int(hx[2:4], 16) & 0xf0 == 0x80:
        return 4
    if (op == 0xeb or op & 0xf0 == 0x70 or op == 0xe3) and n == 2:
        return 1
    return 0


def load(path):
    fs, cur = [], None
    for l in open(path, encoding='utf-8', errors='replace'):
        p = l.rstrip('\n').split('\t')
        if p[0] == 'F':
            cur = Func()
            cur.entry, cur.end, cur.name, cur.ins = int(p[1], 16), int(p[2], 16), p[3], []
            fs.append(cur)
        else:
            a, hx = int(p[0], 16), p[1]
            refs = [(r[0], int(r[1:], 16)) for r in p[2].split(',')] if p[2] else []
            bl = branch_len(hx)
            # key2: the bytes with every relative displacement blanked (inner jumps move when code is inserted)
            k2 = hx[:len(hx) - 2 * bl] + '00' * bl if bl else hx
            tgt = None
            if bl and not any(k == 'c' for k, _ in refs):
                disp = int.from_bytes(bytes.fromhex(hx[len(hx) - 2 * bl:]), 'little', signed=True)
                tgt = a + len(hx) // 2 + disp
            cur.ins.append((a, hx, refs, p[3] if len(p) > 3 else '', k2, tgt))
    fs.sort(key=lambda f: f.entry)
    for i, f in enumerate(fs):
        h = hashlib.blake2b(digest_size=12)
        for x in f.ins:
            h.update(bytes.fromhex(x[1]))
            h.update(b'|')
        f.key = h.digest() if f.ins else b'empty%d' % i
        f.lim = fs[i + 1].entry if i + 1 < len(fs) else f.end + 1
    return fs


def sim(a, b):
    if not a.ins or not b.ins:
        return 0.0
    m = difflib.SequenceMatcher(None, [x[4] for x in a.ins], [x[4] for x in b.ins], autojunk=False)
    return m.ratio()


def pair_gap(A, B, lo=0.35):
    """Order-keeping pairing of two short runs of unmatched functions by similarity."""
    if not A or not B:
        return []
    if len(A) * len(B) > 40000:                                  # a huge gap: compare near neighbours only
        return []
    S = [[sim(a, b) for b in B] for a in A]
    n, m = len(A), len(B)
    best = [[0.0] * (m + 1) for _ in range(n + 1)]
    for i in range(n - 1, -1, -1):
        for j in range(m - 1, -1, -1):
            v = max(best[i + 1][j], best[i][j + 1])
            if S[i][j] >= lo:
                v = max(v, S[i][j] + best[i + 1][j + 1])
            best[i][j] = v
    out, i, j = [], 0, 0
    while i < n and j < m:
        if S[i][j] >= lo and abs(best[i][j] - (S[i][j] + best[i + 1][j + 1])) < 1e-12:
            out.append((A[i], B[j], S[i][j]))
            i, j = i + 1, j + 1
        elif best[i + 1][j] >= best[i][j + 1]:
            i += 1
        else:
            j += 1
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('old')
    ap.add_argument('new')
    ap.add_argument('--out')
    a = ap.parse_args()
    out = a.out or os.path.join(a.new, 'remap')
    os.makedirs(os.path.join(out, 'changed'), exist_ok=True)
    for f in os.listdir(os.path.join(out, 'changed')):
        os.remove(os.path.join(out, 'changed', f))
    po, pn = Pe(os.path.join(a.old, 'FFXiMain.unpacked.dll')), Pe(os.path.join(a.new, 'FFXiMain.unpacked.dll'))
    FO, FN = load(os.path.join(a.old, 'sig_insns.tsv')), load(os.path.join(a.new, 'sig_insns.tsv'))
    # only .text: the packed section's bytes are not code until the entry stub has unpacked them
    FO, FN = [f for f in FO if po.is_code(f.entry)], [f for f in FN if pn.is_code(f.entry)]
    for fs in (FO, FN):
        for i, f in enumerate(fs):
            f.lim = fs[i + 1].entry if i + 1 < len(fs) else f.end + 1
    print('functions: old %d, new %d' % (len(FO), len(FN)))

    # 1. align the two lists in address order on the blanked bytes
    sm = difflib.SequenceMatcher(None, [f.key for f in FO], [f.key for f in FN], autojunk=False)
    pairs, gaps, pi, pj = [], [], 0, 0                          # (old, new, similarity)
    for i, j, n in sm.get_matching_blocks():
        if i > pi or j > pj:
            gaps.append((FO[pi:i], FN[pj:j]))
        for k in range(n):
            pairs.append((FO[i + k], FN[j + k], 1.0))
        pi, pj = i + n, j + n
    exact = len(pairs)
    # 2. pair what is left between the aligned runs by similarity
    lone_o, lone_n = [], []
    for A, B in gaps:
        got = pair_gap(A, B)
        pairs += got
        go, gn = {id(x[0]) for x in got}, {id(x[1]) for x in got}
        lone_o += [f for f in A if id(f) not in go]
        lone_n += [f for f in B if id(f) not in gn]
    # 2b. leftovers that moved out of order: same blanked bytes, unique on both sides
    ko, kn = collections.defaultdict(list), collections.defaultdict(list)
    for f in lone_o:
        ko[f.key].append(f)
    for f in lone_n:
        kn[f.key].append(f)
    moved = [(ko[k][0], kn[k][0], 1.0) for k in ko if len(ko[k]) == 1 and len(kn.get(k, ())) == 1 and len(ko[k][0].ins) >= 6]
    pairs += moved
    mo, mn = {id(x[0]) for x in moved}, {id(x[1]) for x in moved}
    lone_o = [f for f in lone_o if id(f) not in mo]
    lone_n = [f for f in lone_n if id(f) not in mn]
    # 2c. leftovers that matched code points at: a call / pointer in a matched instruction, or a pointer slot whose
    #     neighbours moved together, names the twin (folded stubs the linker placed elsewhere)
    by_o, by_n = {f.entry: f for f in lone_o}, {f.entry: f for f in lone_n}
    slot_o, slot_n = pointer_slots(po), pointer_slots(pn)
    while True:
        cand = collections.defaultdict(collections.Counter)
        for fo, fn, _ in pairs:
            if len(fo.ins) != len(fn.ins):
                continue
            for xo, xn in zip(fo.ins, fn.ins):
                if xo[4] == xn[4] and len(xo[2]) == len(xn[2]):
                    for (k1, t1), (k2, t2) in zip(xo[2], xn[2]):
                        if t1 in by_o and t2 in by_n:
                            cand[t1][t2] += 1
        got = []
        for t1, c in cand.items():
            t2, v = c.most_common(1)[0]
            if all(cand[o].most_common(1)[0][0] != t2 or o == t1 for o in cand) and t2 in by_n:
                got.append((by_o[t1], by_n[t2], sim(by_o[t1], by_n[t2])))
        if not got:
            break
        for g in got:
            by_o.pop(g[0].entry, None)
            by_n.pop(g[1].entry, None)
        pairs += got
        moved += got
    lone_o, lone_n = list(by_o.values()), list(by_n.values())
    pairs.sort(key=lambda p: p[0].entry)
    FM = {p[0].entry: p[1].entry for p in pairs}
    # 2d. identical stubs the linker shuffled: among paired functions with the same blanked bytes, the one whose
    #     calls / jumps go to the twins of this one's is the real twin (`mov ecx, x; jmp f` thunks)
    def code_refs(f, mapped):
        r = []
        for x in f.ins:
            for kind, t in x[2]:
                if po.is_code(t):                                 # .text has the same range test in both builds
                    r.append(FM.get(t) if mapped else t)
        return tuple(r)
    g_old, g_new, cur = collections.defaultdict(list), collections.defaultdict(list), {}
    for fo, fn, _ in pairs:
        if fo.key == fn.key and len(fo.ins) <= 8:
            g_old[fo.key].append(fo)
            g_new[fo.key].append(fn)
            cur[fo.entry] = fn
    for f in lone_o:
        if f.key in g_old:
            g_old[f.key].append(f)
    for f in lone_n:
        if f.key in g_new:
            g_new[f.key].append(f)
    shuffled, repair = 0, {}                                      # old entry -> new function or None (lone)
    for key, olds in g_old.items():
        news = g_new[key]
        if len(olds) < 2 and len(news) < 2:
            continue
        want = {f.entry: code_refs(f, True) for f in olds}
        have = collections.defaultdict(list)
        for g in news:
            have[code_refs(g, False)].append(g)
        nwant = collections.Counter(want.values())
        if all(cur.get(f.entry) is not None and code_refs(cur[f.entry], False) == want[f.entry] for f in olds):
            continue
        fixed, taken = {}, set()
        for f in olds:
            w = want[f.entry]
            if w and None not in w and nwant[w] == 1 and len(have.get(w, ())) == 1:
                fixed[f.entry] = have[w][0]
                taken.add(have[w][0].entry)
        for f in olds:
            if f.entry in fixed:
                if cur.get(f.entry) is not fixed[f.entry]:
                    repair[f.entry] = fixed[f.entry]
                    shuffled += 1
            elif f.entry in cur and cur[f.entry].entry in taken:
                repair[f.entry] = None                            # its twin went to the stub that calls the same
    if repair:
        by_e = {f.entry: f for f in FO}
        keep = [p for p in pairs if p[0].entry not in repair]
        used_n = {p[1].entry for p in keep}
        for e, g in repair.items():
            if g is not None:
                keep.append((by_e[e], g, 1.0))
                used_n.add(g.entry)
        pairs = keep
        pairs.sort(key=lambda p: p[0].entry)
        used_o = {p[0].entry for p in pairs}
        lone_o = [f for f in FO if f.entry not in used_o]
        lone_n = [f for f in FN if f.entry not in used_n]
    if shuffled:
        print('identical stubs re-paired by what they call: %d' % shuffled)
        FM = {p[0].entry: p[1].entry for p in pairs}
    pair_of = {p[0].entry: p for p in pairs}
    starts_o = [f.entry for f in FO]

    # 3. instruction alignment of every pair; every blanked address votes
    votes = collections.defaultdict(collections.Counter)         # old address -> new address counts
    who = collections.defaultdict(list)                          # (old, new) -> [old function entry]
    aligned = {}                                                 # old entry -> [(old index, new index)]
    for fo, fn, s in pairs:
        if fo.key == fn.key:
            al = [(k, k) for k in range(len(fo.ins))]
        else:
            m = difflib.SequenceMatcher(None, [x[4] for x in fo.ins], [x[4] for x in fn.ins], autojunk=False)
            al = [(i + k, j + k) for i, j, n in m.get_matching_blocks() for k in range(n)]
        aligned[fo.entry] = al
        for i, j in al:
            ro, rn = fo.ins[i][2], fn.ins[j][2]
            if len(ro) != len(rn):
                continue
            for (k1, t1), (k2, t2) in zip(ro, rn):
                if k1 == k2:
                    votes[t1][t2] += 1
                    if len(who[(t1, t2)]) < 6:
                        who[(t1, t2)].append(fo.entry)
    amap = {t: c.most_common(1)[0][0] for t, c in votes.items()}
    # instruction-level map of code addresses inside matched functions
    imap = {}
    for fo, fn, s in pairs:
        for i, j in aligned[fo.entry]:
            imap[fo.ins[i][0]] = fn.ins[j][0]

    # an address only one or two instructions use, that moved differently from both of its neighbours while they
    # moved together, is no anchor: the instruction points somewhere else now (it is reported with its function)
    da = sorted(t for t in amap if not po.is_code(t) and po.sec(t) is not None)
    odd = set()
    for x, t, y in zip(da, da[1:], da[2:]):
        if po.sec(x) is po.sec(t) is po.sec(y) and amap[x] - x == amap[y] - y != amap[t] - t and votes[t][amap[t]] <= 2:
            odd.add(t)
    for t in list(amap):
        if t in odd or (not po.is_code(t) and po.sec(t) is None):
            del amap[t]
    data_anchor = sorted(t for t in amap if not po.is_code(t))

    def expect(t):
        """Where old address `t` should be in the new build, or None when nothing says."""
        if po.is_code(t):
            if t in FM:
                return FM[t]
            if t in imap:
                return imap[t]
            k = bisect.bisect_right(starts_o, t) - 1
            if k >= 0:
                g = FO[k]
                p = pair_of.get(g.entry)
                if p and t < g.lim and p[0].key == p[1].key and p[1].lim - p[1].entry == g.lim - g.entry:
                    return p[1].entry + (t - g.entry)             # a table after an untouched function
            return amap.get(t)
        if t in amap:
            return amap[t]
        k = bisect.bisect_right(data_anchor, t)
        if 0 < k < len(data_anchor):
            lo_, hi_ = data_anchor[k - 1], data_anchor[k]
            if amap[lo_] - lo_ == amap[hi_] - hi_:
                return t + amap[lo_] - lo_
        return None

    # 3b. leftovers named only by pointer slots (vtables): the slot's place in the new build holds the twin
    extra = []
    for f in list(lone_o):
        c = collections.Counter()
        for sl in slot_o.get(f.entry, ()):
            e = expect(sl)
            v = pn.u32(e) if e is not None else None
            if v is not None:
                c[v] += 1
        if c:
            t2, v = c.most_common(1)[0]
            g = by_n.get(t2)
            if g is not None and v * 2 > sum(c.values()):
                extra.append((f, g, sim(f, g)))
                by_n.pop(t2)
                by_o.pop(f.entry)
    if extra:
        for fo, fn, sm_ in extra:
            pairs.append((fo, fn, sm_))
            FM[fo.entry] = fn.entry
            pair_of[fo.entry] = (fo, fn, sm_)
            if len(fo.ins) == len(fn.ins):
                al = [(k, k) for k in range(len(fo.ins)) if fo.ins[k][4] == fn.ins[k][4]]
            else:
                m = difflib.SequenceMatcher(None, [x[4] for x in fo.ins], [x[4] for x in fn.ins], autojunk=False)
                al = [(i + k, j + k) for i, j, n in m.get_matching_blocks() for k in range(n)]
            aligned[fo.entry] = al
            for i, j in al:
                imap[fo.ins[i][0]] = fn.ins[j][0]
        moved += extra
        pairs.sort(key=lambda p: p[0].entry)
        lone_o, lone_n = list(by_o.values()), list(by_n.values())

    # 4. classify
    rows, nsame, nrefs, nchg = [], 0, 0, 0
    conflicts = [(t, c) for t, c in votes.items() if len(c) > 1]
    for fo, fn, s in pairs:
        bad = []                                                 # (old insn index, new insn index, why)
        al = aligned[fo.entry]
        amap_i = dict(al)
        for i, j in al:
            xo, xn = fo.ins[i], fn.ins[j]
            if len(xo[2]) != len(xn[2]):
                bad.append((i, j, 'refs'))
                continue
            for (k1, t1), (k2, t2) in zip(xo[2], xn[2]):
                e = expect(t1)
                if k1 != k2 or (e is not None and e != t2):
                    bad.append((i, j, '%x -> %x, expected %s' % (t1, t2, '%x' % e if e else '?')))
            if xo[5] is not None and xn[5] is not None:
                e = imap.get(xo[5]) or expect(xo[5])
                if e is not None and e != xn[5]:
                    bad.append((i, j, 'jump target, expected %x' % e))
        whole = len(al) == len(fo.ins) == len(fn.ins)             # every instruction has its twin
        if whole and not bad:
            st = 'same'
            nsame += 1
        elif whole:
            st = 'refs'
            nrefs += 1
        else:
            st = 'changed'
            nchg += 1
        shift = ''
        if st == 'changed' and not bad and len(fo.ins) == len(fn.ins):
            ds = const_shift(fo, fn)
            if ds is not None:
                st = 'consts'
                shift = ','.join('%+x' % d for d in sorted(ds))
        rows.append((fo, fn, st, s, bad, shift))
        if st != 'same':
            write_diff(os.path.join(out, 'changed', '%08x_%08x.diff' % (fo.entry, fn.entry)), fo, fn, al, bad, st, s)

    with open(os.path.join(out, 'funcmap.tsv'), 'w') as w:
        w.write('# old\tnew\tstatus\tsimilarity\tinsns old\tinsns new\told name\tconstant shifts\n')
        for fo, fn, st, s, bad, shift in rows:
            w.write('%08x\t%08x\t%s\t%.3f\t%d\t%d\t%s\t%s\n' % (fo.entry, fn.entry, st, s, len(fo.ins), len(fn.ins), fo.name, shift))
    callers_n = collections.defaultdict(set)
    for f in FN:
        for x in f.ins:
            for k, t in x[2]:
                callers_n[t].add(f.entry)
    callers_o = collections.defaultdict(set)
    for f in FO:
        for x in f.ins:
            for k, t in x[2]:
                callers_o[t].add(f.entry)
    for name, lone, cal, slots in (('new.tsv', lone_n, callers_n, slot_n), ('removed.tsv', lone_o, callers_o, slot_o)):
        with open(os.path.join(out, name), 'w') as w:
            w.write('# entry\tlast\tinsns\tname\treferenced from (functions)\tpointer slots\n')
            for f in sorted(lone, key=lambda f: f.entry):
                w.write('%08x\t%08x\t%d\t%s\t%s\t%s\n' % (f.entry, f.end, len(f.ins), f.name,
                        ','.join('%08x' % c for c in sorted(cal.get(f.entry, ()))[:12]),
                        ','.join('%08x' % c for c in slots.get(f.entry, [])[:12])))
    with open(os.path.join(out, 'datamap.tsv'), 'w') as w:
        w.write('# old\tnew\tdelta\tvotes\n')
        for t in sorted(amap):
            if not po.is_code(t):
                w.write('%08x\t%08x\t%+x\t%d\n' % (t, amap[t], amap[t] - t, votes[t][amap[t]]))
    with open(os.path.join(out, 'conflicts.tsv'), 'w') as w:
        w.write('# old address\tnew address\tvotes\tfunctions (old entries)\n')
        for t, c in sorted(conflicts):
            for n_, v in c.most_common():
                w.write('%08x\t%08x\t%d\t%s\n' % (t, n_, v, ','.join('%08x' % x for x in who[(t, n_)])))

    # the compact map (addresses only; this one may be committed, see maps/)
    segs = []                                                    # (kind, old start, old end, delta)
    for fo in FO:
        p = pair_of.get(fo.entry)
        if not p:
            continue
        fn = p[1]
        al = aligned[fo.entry]
        rigid = len(fo.ins) == len(fn.ins) and fn.lim - fn.entry == fo.lim - fo.entry and all(
            y[0] - x[0] == fn.entry - fo.entry for x, y in zip(fo.ins, fn.ins))   # every instruction at the same offset
        d = fn.entry - fo.entry
        if rigid:
            if segs and segs[-1][0] == 'code' and segs[-1][2] == fo.entry and segs[-1][3] == d:
                segs[-1] = ('code', segs[-1][1], fo.lim, d)
            else:
                segs.append(('code', fo.entry, fo.lim, d))
        else:
            segs.append(('entry', fo.entry, fo.entry + 1, d))
    for t in data_anchor:
        d = amap[t] - t
        if segs and segs[-1][0] == 'data' and segs[-1][3] == d and po.sec(segs[-1][1]) is po.sec(t):
            segs[-1] = ('data', segs[-1][1], t + 1, d)
        else:
            segs.append(('data', t, t + 1, d))
    with open(os.path.join(out, 'segments.tsv'), 'w') as w:
        w.write('# kind\told start\told end (exclusive)\tdelta to the new build\n'
                '# code: whole functions that moved unchanged (interior addresses move with them)\n'
                '# entry: a function whose instructions moved inside it (only its entry is mapped)\n'
                '# data: runs of referenced data addresses that moved together\n')
        for k, a0, a1, d in segs:
            w.write('%s\t%08x\t%08x\t%+d\n' % (k, a0, a1, d))

    ndiff = data_diff(po, pn, FO, FN, pair_of, amap, data_anchor, expect, os.path.join(out, 'data_diff.tsv'))
    summ = [
        'functions: old %d, new %d' % (len(FO), len(FN)),
        'paired %d (aligned identical %d, moved %d)' % (len(pairs), exact, len(moved)),
        '  ' + ', '.join('%s %d' % kv for kv in sorted(collections.Counter(r[2] for r in rows).items())),
        'only old %d, only new %d' % (len(lone_o), len(lone_n)),
        'data addresses mapped %d, conflicting %d' % (len(data_anchor), len(conflicts)),
        'data regions differing %d' % ndiff,
    ]
    open(os.path.join(out, 'summary.txt'), 'w').write('\n'.join(summ) + '\n')
    print('\n'.join(summ))


def const_shift(fo, fn):
    """The set of differences when two functions differ only in immediates / displacements of otherwise identical
    instructions (None when they differ in more). Compared on the text ExportSig wrote, numbers blanked."""
    import re
    num = re.compile(r'-?0x[0-9a-f]+')
    ds = set()
    for xo, xn in zip(fo.ins, fn.ins):
        if xo[4] == xn[4]:
            continue
        if xo[2] or xn[2]:
            return None
        a, b = xo[3], xn[3]
        if num.sub('N', a) != num.sub('N', b):
            return None
        na, nb = num.findall(a), num.findall(b)
        if xo[5] is not None:                                     # an inner jump: its text is an address
            continue
        for x, y in zip(na, nb):
            if x != y:
                ds.add(int(y, 16) - int(x, 16))
    return ds


def write_diff(path, fo, fn, al, bad, st, s):
    badset = {(i, j): why for i, j, why in bad}
    with open(path, 'w') as w:
        w.write('# %08x (%s) -> %08x: %s, similarity %.3f, %d -> %d instructions\n'
                % (fo.entry, fo.name, fn.entry, st, s, len(fo.ins), len(fn.ins)))
        pi, pj = 0, 0
        for i, j in al + [(len(fo.ins), len(fn.ins))]:
            for k in range(pi, i):
                w.write('-%08x  %s\n' % (fo.ins[k][0], fo.ins[k][3]))
            for k in range(pj, j):
                w.write('+%08x  %s\n' % (fn.ins[k][0], fn.ins[k][3]))
            if i < len(fo.ins):
                why = badset.get((i, j))
                if why:
                    w.write('!%08x  %s\n!%08x  %s      <- %s\n' % (fo.ins[i][0], fo.ins[i][3], fn.ins[j][0], fn.ins[j][3], why))
                elif i == pi and j == pj and i > 0:
                    pass
                else:
                    w.write(' %08x  %s\n' % (fn.ins[j][0], fn.ins[j][3]))
            pi, pj = i + 1, j + 1


def pointer_slots(pe):
    """target -> [slot] for every relocated pointer stored in a data section."""
    out = collections.defaultdict(list)
    for r in pe.relocs:
        s = pe.sec(r)
        if s and s[0] != '.text':
            v = pe.u32(r)
            if v is not None:
                out[v].append(r)
    return out


def data_diff(po, pn, FO, FN, pair_of, amap, anchors, expect, path):
    """Compare initialised data once moved: .rdata / .data between mapped addresses that moved together, and the
    bytes after each untouched function (switch tables). Pointers are compared through the maps."""
    def cmp_run(ao, an, n):
        """Differences of old [ao, ao+n) against new [an, an+n): [(old addr, new addr, old hex, new hex, note)]."""
        for lo_, hi_ in skip:                                  # the import tables: handled by name
            if lo_ <= ao < hi_:
                step = min(n, hi_ - ao)
                return cmp_run(ao + step, an + step, n - step) if n > step else []
            if ao < lo_ < ao + n:
                return cmp_run(ao, an, lo_ - ao) + cmp_run(lo_, an + lo_ - ao, n - (lo_ - ao))
        bo, bn = po.bytes(ao, n), pn.bytes(an, n)
        if bo is None or bn is None:
            return []
        n = min(len(bo), len(bn), n)
        res, k = [], 0
        lo = bisect.bisect_left(po.relocs, ao)
        rel = set()
        while lo < len(po.relocs) and po.relocs[lo] + 4 <= ao + n:
            rel.add(po.relocs[lo] - ao)
            lo += 1
        while k < n:
            if k in rel:
                vo, vn = struct.unpack_from('<I', bo, k)[0], struct.unpack_from('<I', bn, k)[0]
                e = expect(vo)
                isrel = (an + k) in pn.relset
                if not isrel or (e is not None and e != vn) or (e is None and (vn - vo) != (an - ao) and po.sec(vo) is not None and (pn.sec(vn) is None or po.sec(vo)[0] != pn.sec(vn)[0])):
                    res.append((ao + k, an + k, '%08x' % vo, '%08x' % vn, 'pointer' + ('' if e is None else ', expected %08x' % e)))
                k += 4
                continue
            if bo[k] != bn[k]:
                j = k
                while j < n and j not in rel and (bo[j] != bn[j] or (j + 1 < n and j + 1 not in rel and bo[j + 1] != bn[j + 1])) and j - k < 64:
                    j += 1
                res.append((ao + k, an + k, bo[k:j].hex(), bn[k:j].hex(), ''))
                k = j
                continue
            k += 1
        return res

    out = []
    # imports: the tables hold RVAs that move with the section; compare the names behind each slot instead
    io, rgo = po.imports()
    inn, rgn = pn.imports()
    skip = sorted(rgo)
    for slot, name in sorted(io.items()):
        e = expect(slot)
        if e is None or inn.get(e) != name:
            out.append((slot, e or 0, '', '', 'import %s: new slot holds %s' % (name, inn.get(e) if e else 'nothing mapped')))
    gone = set(inn.values()) - set(io.values())
    for slot, name in sorted(inn.items()):
        if name in gone:
            out.append((0, slot, '', '', 'new import %s' % name))

    def holes(pe, F):
        """function entry -> [(offset from entry, length)]: the bytes up to the next function no instruction covers,
        filler at the end dropped."""
        t = next(x for x in pe.secs if x[0] == '.text')
        cov = bytearray(t[2])
        for f in F:
            for x in f.ins:
                o, n = x[0] - t[1], len(x[1]) // 2
                if 0 <= o < t[2]:
                    cov[o:o + n] = b'\1' * n
        res = {}
        for f in F:
            runs, i, lim = [], f.entry - t[1], min(f.lim - t[1], t[2])
            while i < lim:
                if cov[i]:
                    i += 1
                    continue
                j = i
                while j < lim and not cov[j]:
                    j += 1
                b = pe.bytes(t[1] + i, j - i)
                keep = len(b.rstrip(b'\xcc\x90\x00')) if j == lim else len(b)
                lead = len(b[:keep]) - len(b[:keep].lstrip(b'\xcc\x90'))
                if keep - lead > 0:
                    runs.append((i + lead - (f.entry - t[1]), keep - lead))
                i = j
            res[f.entry] = runs
        return res

    # tables kept in .text, by the function in front of them
    ho, hn = holes(po, FO), holes(pn, FN)
    for fo in FO:
        p = pair_of.get(fo.entry)
        if not p:
            continue
        fn = p[1]
        ro, rn = ho[fo.entry], hn[fn.entry]
        if not ro and not rn:
            continue
        if p[0].key != p[1].key or len(fo.ins) != len(fn.ins):   # changed code: compare the tables in order
            if [n for _, n in ro] != [n for _, n in rn]:
                out.append((fo.entry, fn.entry, '', '', 'tables in changed function: old %s, new %s'
                            % (' '.join('+%x:%d' % r for r in ro), ' '.join('+%x:%d' % r for r in rn))))
                continue
        elif ro != rn:
            # the same function with its table bytes decoded differently (a table entry read as an instruction
            # in one build): compare the whole tail byte for byte
            lo_ = min([o for o, _ in ro] + [o for o, _ in rn])
            if fo.lim - fo.entry == fn.lim - fn.entry:
                d = cmp_run(fo.entry + lo_, fn.entry + lo_, fo.lim - fo.entry - lo_)
                out += [(x[0], x[1], x[2], x[3], (x[4] + ' (tail of function %08x)' % fo.entry).strip()) for x in d]
            else:
                out.append((fo.entry, fn.entry, '', '', 'tables after function: old %s, new %s'
                            % (' '.join('+%x:%d' % r for r in ro), ' '.join('+%x:%d' % r for r in rn))))
            continue
        for (oo, n), (on, _) in zip(ro, rn):
            out += cmp_run(fo.entry + oo, fn.entry + on, n)
    # data sections between anchors
    resized = 0
    for a0, a1 in zip(anchors, anchors[1:]):
        so = po.sec(a0)
        if so is None or po.sec(a1) is not so:
            continue
        d0, d1 = amap[a0] - a0, amap[a1] - a1
        if d0 == d1:
            out += cmp_run(a0, a0 + d0, a1 - a0)
        else:
            resized += 1
            no, nn = a1 - a0, amap[a1] - amap[a0]
            out.append((a0, amap[a0], '', '', 'resized: old %d bytes to next mapped address %08x, new %d bytes to %08x'
                        % (no, a1, nn, amap[a1])))
            bo, bn = po.bytes(a0, no), pn.bytes(amap[a0], nn)
            if bo is None or bn is None or len(bo) != no or len(bn) != nn:
                out.append((a0, amap[a0], '', '', '  (not initialised data: nothing to compare)'))
                continue
            # pointers are compared as what they point at
            def toks(pe, base, b, mapper):
                res, k = [], 0
                while k < len(b):
                    if base + k in pe.relset and k + 4 <= len(b):
                        v = struct.unpack_from('<I', b, k)[0]
                        res.append(('p', mapper(v)))
                        k += 4
                    else:
                        res.append(b[k])
                        k += 1
                return res
            to = toks(po, a0, bo, lambda v: expect(v) or ('old', v))
            tn = toks(pn, amap[a0], bn, lambda v: v)
            def size(ts):
                return sum(4 if isinstance(t, tuple) else 1 for t in ts)
            def show(ts):
                return ''.join('<%s>' % ('%08x' % t[1] if isinstance(t[1], int) else 'old %08x' % t[1][1]) if isinstance(t, tuple) else '%02x' % t for t in ts[:200])
            m = difflib.SequenceMatcher(None, to, tn, autojunk=False)
            for tag, i1, i2, j1, j2 in m.get_opcodes():
                if tag != 'equal':
                    out.append((a0 + size(to[:i1]), amap[a0] + size(tn[:j1]), show(to[i1:i2]), show(tn[j1:j2]),
                                '  %s: %d bytes -> %d bytes' % (tag, size(to[i1:i2]), size(tn[j1:j2]))))
    with open(path, 'w') as w:
        w.write('# old address\tnew address\told bytes\tnew bytes\tnote\n')
        for r in sorted(out, key=lambda r: (r[0] or r[1], r[4].startswith(' '))):
            w.write('%08x\t%08x\t%s\t%s\t%s\n' % r)
    return len(out)


if __name__ == '__main__':
    main()
