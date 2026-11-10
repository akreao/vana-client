#!/usr/bin/env python3
"""Move names.tsv and names_manual.tsv to a new FFXiMain.dll build.

    python3 port_names.py [maps/<older build>.tsv]      (default: the newest map)

Every address goes through the map remap.py wrote (copied to maps/). Names whose function is gone in the new build are
dropped and listed. Run once per build move, then ./ri_named.sh (build_names.py re-derives what it can from the DLL
and keeps the rest of these rows).
"""
import os, sys
import addrmap
H = os.path.dirname(os.path.abspath(__file__))
m = addrmap.Map(sys.argv[1]) if len(sys.argv) > 1 else addrmap.load(H)
if m is None:
    sys.exit('no map in maps/')
for name in ('names.tsv', 'names_manual.tsv'):
    p = os.path.join(H, name)
    if not os.path.exists(p):
        continue
    out, moved, lost = [], 0, []
    for l in open(p):
        if l.startswith('#') or not l.strip():
            out.append(l)
            continue
        f = l.rstrip('\n').split('\t')
        a = int(f[0].replace('0x', ''), 16)
        b = m.fwd(a)
        if b is None:
            lost.append(l.rstrip('\n'))
            continue
        moved += b != a
        out.append('\t'.join(['%08x' % b] + f[1:]) + '\n')
    open(p, 'w').write(''.join(out))
    print('%s: %d rows, %d moved, %d dropped' % (name, len(out), moved, len(lost)))
    for l in lost:
        print('  dropped:', l)
