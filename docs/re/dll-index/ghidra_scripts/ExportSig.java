// Exports every function's instructions in a form two builds of the DLL can be compared by (remap.py).
// Each instruction's bytes are written with everything that moves between builds blanked: the four bytes at each base
// relocation (absolute addresses), and the displacement of a relative call / jump that leaves the function. What was
// blanked is listed beside it, so the tool can check that it points at the matching place in the other build.
// Script arg: the work dir (writes sig_insns.tsv); else FFXI_RE_DIR. Run after FixVt (and after the rizin list
// has been applied, see ri.sh).
//   sig_insns.tsv: "F <entry> <last body address> <name>" per function, then per instruction
//                  "<addr> <masked hex> <refs: a=absolute, c=leaving flow; comma list> <text>"
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.symbol.*;
import java.io.*;
import java.util.*;
public class ExportSig extends GhidraScript {
  public void run() throws Exception {
    String D = FixVt.dir(getScriptArgs());
    long[] rel = FixVt.relocs(currentProgram).stream().mapToLong(Long::longValue).toArray();
    FunctionManager fm = currentProgram.getFunctionManager();
    Listing li = currentProgram.getListing();
    int nf = 0; long ni = 0;
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "sig_insns.tsv"), 1 << 20))) {
      for (Function f : fm.getFunctions(true)) {
        if (f.isExternal()) continue;
        AddressSetView body = f.getBody();
        w.println("F\t" + f.getEntryPoint() + "\t" + body.getMaxAddress() + "\t" + f.getName());
        nf++;
        InstructionIterator it = li.getInstructions(body, true);
        while (it.hasNext()) {
          Instruction in = it.next();
          long a = in.getAddress().getOffset();
          byte[] b = in.getBytes();
          StringBuilder refs = new StringBuilder();
          int k = Arrays.binarySearch(rel, a);
          if (k < 0) k = -k - 1;
          for (; k < rel.length && rel[k] + 4 <= a + b.length; k++) {
            int o = (int) (rel[k] - a);
            long v = (b[o] & 0xffL) | (b[o + 1] & 0xffL) << 8 | (b[o + 2] & 0xffL) << 16 | (b[o + 3] & 0xffL) << 24;
            b[o] = b[o + 1] = b[o + 2] = b[o + 3] = 0;
            if (refs.length() > 0) refs.append(',');
            refs.append("a").append(Long.toHexString(v));
          }
          FlowType ft = in.getFlowType();
          if (ft.isCall() || ft.isJump()) {
            int op = b[0] & 0xff, n = 0;
            if (op == 0xe8 || op == 0xe9) n = 4;
            else if (op == 0x0f && b.length == 6 && (b[1] & 0xf0) == 0x80) n = 4;
            else if (op == 0xeb || (op & 0xf0) == 0x70 || op == 0xe3) n = 1;
            if (n > 0 && b.length == (op == 0x0f ? 2 : 1) + n) {
              for (Address t : in.getFlows()) {
                if (ft.isCall() || !body.contains(t)) {
                  for (int j = b.length - n; j < b.length; j++) b[j] = 0;
                  if (refs.length() > 0) refs.append(',');
                  refs.append("c").append(t);
                }
              }
            }
          }
          StringBuilder hx = new StringBuilder();
          for (byte x : b) hx.append(String.format("%02x", x & 0xff));
          w.println(in.getAddress() + "\t" + hx + "\t" + refs + "\t" + in);
          ni++;
        }
      }
    }
    println("ExportSig: " + nf + " functions, " + ni + " instructions");
  }
}
