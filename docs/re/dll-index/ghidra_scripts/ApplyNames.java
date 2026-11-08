// Renames functions from names.tsv + names_manual.tsv (manual wins), SourceType.USER_DEFINED.
// Where no function starts at the address (vtable/OnKey/event-op targets) it creates one; if Ghidra
// cannot (overlap it will not fix), it puts a USER_DEFINED label there instead.
// Run NOT read-only, and only on the copy ghidra_named/ (ri_named.sh).
// Script arg: the dir holding names.tsv / names_manual.tsv (ri_named.sh passes the script folder).
import ghidra.app.script.GhidraScript;
import ghidra.app.cmd.function.CreateFunctionCmd;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.symbol.*;
import java.nio.file.*;
import java.util.*;
public class ApplyNames extends GhidraScript {
  public void run() throws Exception {
    String[] args = getScriptArgs();
    if (args.length < 1) throw new IllegalArgumentException("pass the dir holding names.tsv as script arg");
    String D = args[0].endsWith("/") ? args[0] : args[0] + "/";
    Map<String, String> m = new TreeMap<>();
    for (String f : new String[] {"names.tsv", "names_manual.tsv"}) {
      Path p = Paths.get(D + f);
      if (!Files.exists(p)) continue;
      for (String l : Files.readAllLines(p)) {
        if (l.startsWith("#") || l.trim().isEmpty()) continue;
        String[] x = l.split("\t");
        if (x.length >= 2) m.put(x[0].replace("0x", ""), x[1]);
      }
    }
    FunctionManager fm = currentProgram.getFunctionManager();
    SymbolTable st = currentProgram.getSymbolTable();
    int renamed = 0, created = 0, labels = 0, failed = 0;
    for (Map.Entry<String, String> e : m.entrySet()) {
      Address a = toAddr(Long.parseLong(e.getKey(), 16));
      String n = e.getValue();
      try {
        Function f = fm.getFunctionAt(a);
        if (f == null) {
          if (getInstructionAt(a) == null) disassemble(a);
          CreateFunctionCmd c = new CreateFunctionCmd(a);
          if (c.applyTo(currentProgram, monitor)) { f = fm.getFunctionAt(a); if (f != null) created++; }
        }
        if (f != null) { f.setName(n, SourceType.USER_DEFINED); renamed++; }
        else { Symbol s = st.createLabel(a, n, SourceType.USER_DEFINED); s.setPrimary(); labels++; }
      } catch (Exception ex) { failed++; println("fail " + e.getKey() + " " + n + ": " + ex.getMessage()); }
    }
    println("ApplyNames: " + m.size() + " names, renamed " + renamed + " (created " + created + "), labels " + labels + ", failed " + failed);
  }
}
