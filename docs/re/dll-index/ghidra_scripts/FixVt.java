// Defines the functions that are reached only through a stored pointer (vtable slots, callback tables, `push offset
// fn`), which Ghidra's auto-analysis leaves as loose code or undefined bytes. Every absolute pointer in the DLL has a
// base relocation, so the .reloc table lists exactly the places to look: a relocated pointer into an executable block
// that is not inside any function gets a function. Pointers stored inside .text itself (switch tables) are skipped:
// their targets are case labels, not functions.
// A second pass then sweeps .text for code nothing reaches at all (see sweep()).
// Script arg: the work dir (writes fixvt.tsv: entry <TAB> first referencing slot, or "sweep"); else FFXI_RE_DIR.
// Run before ExportIndex / ExportSig. Read-only runs keep the functions for that run only.
import ghidra.app.script.GhidraScript;
import ghidra.app.cmd.function.CreateFunctionCmd;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.mem.*;
import java.io.*;
import java.util.*;
public class FixVt extends GhidraScript {
  static String dir(String[] args) {
    String d = args.length > 0 ? args[0] : System.getenv("FFXI_RE_DIR");
    if (d == null || d.isEmpty()) throw new IllegalArgumentException("pass the work dir as script arg or set FFXI_RE_DIR");
    return d.endsWith("/") ? d : d + "/";
  }
  /** Every HIGHLOW slot of the PE base relocation table, ascending. */
  static List<Long> relocs(ghidra.program.model.listing.Program p) throws Exception {
    Memory mem = p.getMemory();
    MemoryBlock rb = mem.getBlock(".reloc");
    if (rb == null) throw new IllegalStateException("no .reloc block");
    long base = p.getImageBase().getOffset();
    byte[] b = new byte[(int) rb.getSize()];
    rb.getBytes(rb.getStart(), b);
    java.nio.ByteBuffer bb = java.nio.ByteBuffer.wrap(b).order(java.nio.ByteOrder.LITTLE_ENDIAN);
    List<Long> out = new ArrayList<>();
    int o = 0;
    while (o + 8 <= b.length) {
      long page = bb.getInt(o) & 0xffffffffL; int size = bb.getInt(o + 4);
      if (size < 8) break;
      for (int k = o + 8; k + 2 <= o + size && k + 2 <= b.length; k += 2) {
        int e = bb.getShort(k) & 0xffff;
        if (e >> 12 == 3) out.add(base + page + (e & 0xfff));
      }
      o += size;
    }
    Collections.sort(out);
    return out;
  }
  public void run() throws Exception {
    String D = dir(getScriptArgs());
    Memory mem = currentProgram.getMemory();
    FunctionManager fm = currentProgram.getFunctionManager();
    Listing li = currentProgram.getListing();
    TreeMap<Long, Long> want = new TreeMap<>();          // target -> first slot
    int inText = 0, tables = 0;
    List<Long> rl = relocs(currentProgram);
    Set<Long> rs = new HashSet<>(rl);
    for (long slot : rl) {
      Address sa = toAddr(slot);
      MemoryBlock sb = mem.getBlock(sa);
      if (sb == null || !sb.isInitialized()) continue;
      long t = mem.getInt(sa) & 0xffffffffL;
      Address ta = toAddr(t);
      MemoryBlock tb = mem.getBlock(ta);
      if (tb == null || !tb.isExecute() || !tb.isInitialized()) continue;
      if (fm.getFunctionContaining(ta) != null) continue;
      Instruction si = sb.isExecute() ? li.getInstructionContaining(sa) : null;
      if (sb.isExecute() && si == null) { inText++; continue; }
      // a table kept in .text, not code: defined data, a list of pointers, or the base of a memory operand
      // (`jmp [table + eax*4]`, `movzx eax, byte ptr [table + eax]`)
      if (li.getDefinedDataContaining(ta) != null || rs.contains(t) || (si != null && memOperand(si, t))) { tables++; continue; }
      want.putIfAbsent(t, slot);
    }
    int created = 0, failed = 0;
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "fixvt.tsv")))) {
      for (Map.Entry<Long, Long> e : want.entrySet()) {
        Address a = toAddr(e.getKey());
        if (fm.getFunctionContaining(a) != null) continue;         // swallowed by one made a moment ago
        Instruction in = li.getInstructionContaining(a);
        if (in != null && !in.getAddress().equals(a)) { failed++; continue; }   // points into the middle of an instruction
        if (in == null) disassemble(a);
        if (li.getInstructionAt(a) == null) { failed++; continue; }
        CreateFunctionCmd c = new CreateFunctionCmd(a);
        if (c.applyTo(currentProgram, monitor) && fm.getFunctionAt(a) != null) {
          created++; w.println(a + "\t" + Long.toHexString(e.getValue()));
        } else failed++;
      }
    }
    println("FixVt: " + want.size() + " pointer-only targets, created " + created + ", failed " + failed
        + ", skipped " + inText + " stored inside code (switch tables) and " + tables + " pointing at tables in code");
    sweep(D);
  }
  /** Whether `in` uses address `t` as the base of a memory operand (then `t` is data). */
  static boolean memOperand(Instruction in, long t) {
    String h = "0x" + Long.toHexString(t);
    for (int i = 0; i < in.getNumOperands(); i++) {
      String r = in.getDefaultOperandRepresentation(i).toLowerCase();
      if (r.contains(h) && r.contains("[")) return true;
    }
    return false;
  }
  /** Length of the compiler's alignment filler at `p` (0 when the bytes are none of its forms). */
  static int filler(Memory mem, Address p) {
    int[][] pat = {
      {0x8d, 0xa4, 0x24, 0, 0, 0, 0}, {0x8d, 0x9b, 0, 0, 0, 0}, {0x8d, 0x80, 0, 0, 0, 0}, {0x8d, 0x89, 0, 0, 0, 0},
      {0x8d, 0xb6, 0, 0, 0, 0}, {0x8d, 0xbf, 0, 0, 0, 0}, {0x8d, 0x92, 0, 0, 0, 0},
      {0x05, 0, 0, 0, 0}, {0x8d, 0x64, 0x24, 0}, {0x8d, 0x74, 0x26, 0}, {0x8d, 0x7c, 0x27, 0},
      {0x8d, 0x49, 0}, {0x8d, 0x40, 0}, {0x8d, 0x76, 0}, {0x8d, 0x7f, 0}, {0x8d, 0x52, 0}, {0x8d, 0x5b, 0},
      {0x8b, 0xff}, {0x8b, 0xc0}, {0x8b, 0xc9}, {0x8b, 0xd2}, {0x8b, 0xf6}, {0x8b, 0xdb}, {0x90}, {0xcc},
    };
    for (int[] q : pat) {
      boolean ok = true;
      try { for (int i = 0; i < q.length && ok; i++) ok = (mem.getByte(p.add(i)) & 0xff) == q[i]; }
      catch (Exception e) { ok = false; }
      if (ok) return q.length;
    }
    return 0;
  }
  /** Second pass: code no pointer or call reaches (dead or debug functions). Walks .text; whatever is not in a
   *  function, not defined data, not padding (cc / 90 / 00) and not a relocated pointer (a switch table) is
   *  disassembled where it starts and made a function. Appended to fixvt.tsv with "sweep" in the slot column. */
  void sweep(String D) throws Exception {
    Memory mem = currentProgram.getMemory();
    FunctionManager fm = currentProgram.getFunctionManager();
    Listing li = currentProgram.getListing();
    MemoryBlock text = mem.getBlock(".text");
    if (text == null) { println("FixVt sweep: no .text block"); return; }
    List<Long> rl = relocs(currentProgram);
    Set<Long> rel = new HashSet<>(rl);
    Address p = text.getStart(), end = text.getEnd();
    int created = 0, failed = 0, data = 0; long junk = 0;
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "fixvt.tsv", true)))) {
      while (p.compareTo(end) <= 0) {
        Function f = fm.getFunctionContaining(p);
        if (f != null) { p = f.getBody().getRangeContaining(p).getMaxAddress().add(1); continue; }
        Data d = li.getDefinedDataContaining(p);
        if (d != null) { p = d.getMaxAddress().add(1); continue; }
        Instruction in = li.getInstructionContaining(p);
        boolean fresh = in == null;
        if (in == null) {
          int b = mem.getByte(p) & 0xff;
          if (b == 0xcc || b == 0x90 || b == 0x00) { p = p.add(1); continue; }
          if (rel.contains(p.getOffset())) { p = p.add(4); continue; }
          // alignment filler in front of a pointer table is not a function start
          Address q = p; int n;
          while ((n = filler(mem, q)) > 0 && q.subtract(p) < 16) q = q.add(n);
          if (!q.equals(p) && rel.contains(q.getOffset()) && li.getInstructionContaining(q) == null) { p = q; continue; }
          disassemble(p);
          in = li.getInstructionAt(p);
          if (in == null) { junk++; p = p.add(1); continue; }
        } else {
          if (!in.getAddress().equals(p)) { p = in.getMaxAddress().add(1); continue; }
          String m = in.getMnemonicString();
          if (m.equals("NOP") || m.equals("INT3")) { p = in.getMaxAddress().add(1); continue; }
        }
        CreateFunctionCmd c = new CreateFunctionCmd(p);
        Function g = c.applyTo(currentProgram, monitor) ? fm.getFunctionAt(p) : null;
        if (g != null && fresh) {
          // data decoded as code gives itself away: a relocated pointer that is not the tail of one instruction
          long badAt = -1;
          for (AddressRange r : g.getBody()) {
            int k = Collections.binarySearch(rl, r.getMinAddress().getOffset());
            if (k < 0) k = -k - 1;
            for (; k < rl.size() && rl.get(k) <= r.getMaxAddress().getOffset(); k++) {
              Address ra = toAddr(rl.get(k));
              Instruction ic = li.getInstructionContaining(ra);
              if (ic == null || ic.getAddress().equals(ra) || ra.add(3).compareTo(ic.getMaxAddress()) > 0) { badAt = rl.get(k); break; }
            }
            if (badAt >= 0) break;
          }
          if (badAt >= 0 && badAt >= p.getOffset()) {
            AddressSet body = new AddressSet(g.getBody());
            fm.removeFunction(p);
            li.clearCodeUnits(body.getMinAddress(), body.getMaxAddress(), false);
            data++;
            p = toAddr(badAt + 4);
            continue;
          }
        }
        if (g != null) {
          created++; w.println(p + "\tsweep");
          p = g.getBody().getRangeContaining(p).getMaxAddress().add(1);
        } else { failed++; p = in.getMaxAddress().add(1); }
      }
    }
    println("FixVt sweep: created " + created + ", failed " + failed + ", undecodable bytes " + junk + ", data runs left alone " + data);
  }
}
