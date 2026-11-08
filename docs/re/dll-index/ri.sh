#!/bin/sh
# Build the grep index (all_decomp_full.c, xrefs.tsv, strrefs.tsv, funcs_full.tsv) in the work dir.
# The first run imports the DLL into a new Ghidra project (with auto-analysis, slow); later runs reuse it read-only.
#   FFXI_RE_DIR   work dir (default: ./work next to this script); must hold FFXiMain.unpacked.dll
#   FFXI_DLL      the unpacked DLL (default: $FFXI_RE_DIR/FFXiMain.unpacked.dll)
#   GHIDRA_HOME   Ghidra install (default: /opt/ghidra)
set -e
H=$(cd "$(dirname "$0")" && pwd)
W=${FFXI_RE_DIR:-$H/work}
mkdir -p "$W"; W=$(cd "$W" && pwd)
DLL=${FFXI_DLL:-$W/FFXiMain.unpacked.dll}
G=${GHIDRA_HOME:-/opt/ghidra}/support/analyzeHeadless
[ -f "$DLL" ] || { echo "missing $DLL (bring an already unpacked FFXiMain.dll, see README.md)" >&2; exit 1; }
if [ ! -f "$W/ghidra/ffxi.gpr" ]; then
  echo "creating Ghidra project $W/ghidra (import + auto-analysis; this takes a while)"
  mkdir -p "$W/ghidra"
  "$G" "$W/ghidra" ffxi -import "$DLL" 2>&1 | tee "$W/ghidra_import.log" | grep -E 'IMPORTING|REPORT|ERROR' || true
  [ -f "$W/ghidra/ffxi.gpr" ] || { echo "Ghidra import failed, see $W/ghidra_import.log" >&2; exit 1; }
fi
# drop the previous index first, so a failed run can't leave old files looking current
rm -f "$W/all_decomp_full.c" "$W/funcs_full.tsv" "$W/xrefs.tsv" "$W/strrefs.tsv"
rc=0
"$G" "$W/ghidra" ffxi -process "$(basename "$DLL")" -noanalysis -readOnly \
  -scriptPath "$H/ghidra_scripts" -postScript ExportIndex.java "$W" >"$W/exportindex.log" 2>&1 || rc=$?
grep -E 'ExportIndex|ERROR' "$W/exportindex.log" || true
if [ $rc -ne 0 ] || ! grep -q 'ExportIndex.java> done' "$W/exportindex.log" || [ ! -s "$W/all_decomp_full.c" ]; then
  echo "export failed (exit $rc), see $W/exportindex.log" >&2; exit 1
fi
