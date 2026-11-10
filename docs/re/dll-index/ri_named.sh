#!/bin/sh
# Apply names.tsv + names_manual.tsv to a COPY of the Ghidra project (work/ghidra_named) and re-export the grep index
# from it, so all_decomp_full.c shows names at call sites. The previous index is kept as *.prev.*.
#   ./ri_named.sh          refresh ghidra_named from ghidra/ first (default)
#   ./ri_named.sh --keep   reuse ghidra_named as it is (names are re-applied; renames are idempotent)
# Needs ./ri.sh to have run once. Same FFXI_RE_DIR / FFXI_DLL / GHIDRA_HOME settings as ri.sh.
set -e
H=$(cd "$(dirname "$0")" && pwd)
W=${FFXI_RE_DIR:-$H/work}
W=$(cd "$W" && pwd)
DLL=${FFXI_DLL:-$W/FFXiMain.unpacked.dll}
G=${GHIDRA_HOME:-/opt/ghidra}/support/analyzeHeadless
export FFXI_RE_DIR="$W"
cd "$W"
IDX="all_decomp_full.c funcs_full.tsv xrefs.tsv strrefs.tsv"
[ -d ghidra ] || { echo "run ./ri.sh first" >&2; exit 1; }
for f in $IDX; do [ -s "$f" ] || { echo "index incomplete ($f missing): run ./ri.sh first" >&2; exit 1; }; done
[ -f funcs_full.orig.tsv ] || cp -p funcs_full.tsv funcs_full.orig.tsv
python3 "$H/build_names.py"
if [ "$1" != "--keep" ] || [ ! -d ghidra_named ]; then rm -rf ghidra_named; cp -a ghidra ghidra_named; fi
prev() { echo "$1" | sed 's/\.\([a-z]*\)$/.prev.\1/'; }
restore() { for f in $MOVED; do mv -f "$(prev "$f")" "$f"; done; }
ok=0; MOVED=""
trap '[ $ok -eq 1 ] || restore' EXIT                    # any failure or interrupt puts the previous complete index back
trap 'exit 1' INT TERM HUP
for f in $IDX; do mv -f "$f" "$(prev "$f")"; MOVED="$MOVED $f"; done      # move, so a failed run can't leave old files looking current
rc=0
"$G" "$W/ghidra_named" ffxi -process "$(basename "$DLL")" -noanalysis \
  -scriptPath "$H/ghidra_scripts" -postScript FixVt.java "$W" -postScript ApplyNames.java "$H" -postScript ExportIndex.java "$W" \
  >"$W/exportindex_named.log" 2>&1 || rc=$?
grep -E 'FixVt|ApplyNames|ExportIndex|ERROR' "$W/exportindex_named.log" || true
if [ $rc -ne 0 ] || ! grep -q 'ExportIndex.java> done' "$W/exportindex_named.log"; then
  echo "named export failed (exit $rc), previous index restored; see $W/exportindex_named.log" >&2; exit 1
fi
# keep the rizin/ghidra source column from the previous export; functions ApplyNames created are 'named'
python3 - "$W" <<'PY'
import sys; D = sys.argv[1] + '/'
old = {l.split('\t')[0]: l.rstrip('\n').split('\t')[3] for l in open(D + 'funcs_full.prev.tsv')}
rows = [l.rstrip('\n').split('\t') for l in open(D + 'funcs_full.tsv')]
with open(D + 'funcs_full.tsv', 'w') as w:
    for r in rows: r[3] = old.get(r[0], 'named'); w.write('\t'.join(r) + '\n')
PY
ok=1
