// One-time full export for grep-only DLL research. Run read-only; nothing is saved to the project.
// Script arg: the work dir to write into (ri.sh passes it); else the FFXI_RE_DIR environment variable.
// 1. Defines every function rizin found (funcs.txt, optional: first column of `rizin -A -c aflq`) that Ghidra is missing.
// 2. Decompiles every function (parallel) -> all_decomp_full.c  ("//@@FUNC <addr>" blocks, same format as all_decomp.c)
// 3. xrefs.tsv   : from_addr  from_func  to_addr  to_func  type
// 4. strrefs.tsv : str_addr  from_addr  from_func  string
// 5. funcs_full.tsv : entry  end  name  source(ghidra|rizin|vtable|sweep; the last two defined by FixVt.java)
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.symbol.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
public class ExportIndex extends GhidraScript {
  String D;
  String fa(Function f) { return f == null ? "-" : f.getEntryPoint().toString(); }
  String esc(String s) { return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r"); }
  static String dir(String[] args) {
    String d = args.length > 0 ? args[0] : System.getenv("FFXI_RE_DIR");
    if (d == null || d.isEmpty()) throw new IllegalArgumentException("pass the work dir as script arg or set FFXI_RE_DIR");
    return d.endsWith("/") ? d : d + "/";
  }
  public void run() throws Exception {
    D = dir(getScriptArgs());
    FunctionManager fm = currentProgram.getFunctionManager();
    Set<String> added = new HashSet<>();
    int tried = 0;
    Path ft = Paths.get(D + "funcs.txt");
    if (!Files.exists(ft)) println("no funcs.txt in " + D + "; skipping the rizin function list (step 1)");
    for (String l : Files.exists(ft) ? Files.readAllLines(ft) : new ArrayList<String>()) {
      l = l.trim(); if (!l.startsWith("0x")) continue;
      l = l.split("\\s+")[0];
      Address a = toAddr(Long.parseLong(l.substring(2), 16));
      if (fm.getFunctionContaining(a) != null) continue;
      tried++;
      try { disassemble(a); Function f = createFunction(a, null); if (f != null) added.add(f.getEntryPoint().toString()); }
      catch (Exception e) { }
    }
    println("missing tried " + tried + ", created " + added.size());
    Set<String> vt = new HashSet<>();                 // FixVt's functions, when it ran just before (ri.sh)
    Path fv = Paths.get(D + "fixvt.tsv");
    Set<String> sw = new HashSet<>();
    if (Files.exists(fv)) for (String l : Files.readAllLines(fv)) (l.endsWith("\tsweep") ? sw : vt).add(l.split("\t")[0]);

    List<Function> fs = new ArrayList<>();
    for (Function f : fm.getFunctions(true)) fs.add(f);
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "funcs_full.tsv")))) {
      for (Function f : fs) w.println(fa(f) + "\t" + f.getBody().getMaxAddress() + "\t" + f.getName() + "\t" + (added.contains(fa(f)) ? "rizin" : vt.contains(fa(f)) ? "vtable" : sw.contains(fa(f)) ? "sweep" : "ghidra"));
    }

    ReferenceManager rm = currentProgram.getReferenceManager();
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "xrefs.tsv")))) {
      AddressIterator it = rm.getReferenceSourceIterator(currentProgram.getMinAddress(), true);
      while (it.hasNext()) {
        Address from = it.next();
        for (Reference r : rm.getReferencesFrom(from)) {
          Address to = r.getToAddress();
          if (!to.isMemoryAddress()) continue;
          w.println(from + "\t" + fa(fm.getFunctionContaining(from)) + "\t" + to + "\t" + fa(fm.getFunctionContaining(to)) + "\t" + r.getReferenceType());
        }
      }
    }
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new OutputStreamWriter(new FileOutputStream(D + "strrefs.tsv"), "UTF-8")))) {
      DataIterator di = currentProgram.getListing().getDefinedData(true);
      while (di.hasNext()) {
        Data d = di.next();
        if (!d.hasStringValue()) continue;
        String s = esc(String.valueOf(d.getValue()));
        boolean any = false;
        for (Reference r : rm.getReferencesTo(d.getAddress())) {
          any = true;
          w.println(d.getAddress() + "\t" + r.getFromAddress() + "\t" + fa(fm.getFunctionContaining(r.getFromAddress())) + "\t" + s);
        }
        if (!any) w.println(d.getAddress() + "\t-\t-\t" + s);
      }
    }
    println("xrefs/strrefs done; decompiling " + fs.size());

    int T = Math.max(2, Runtime.getRuntime().availableProcessors() - 1);
    ExecutorService ex = Executors.newFixedThreadPool(T);
    String[] out = new String[fs.size()];
    ThreadLocal<DecompInterface> tl = ThreadLocal.withInitial(() -> { DecompInterface x = new DecompInterface(); x.openProgram(currentProgram); return x; });
    List<Future<?>> jobs = new ArrayList<>();
    for (int i = 0; i < fs.size(); i++) {
      final int k = i;
      jobs.add(ex.submit(() -> {
        try {
          DecompileResults r = tl.get().decompileFunction(fs.get(k), 120, monitor);
          out[k] = r.getDecompiledFunction() != null ? r.getDecompiledFunction().getC() : "// decompile failed: " + r.getErrorMessage() + "\n";
        } catch (Exception e) { out[k] = "// decompile error: " + e + "\n"; }
      }));
    }
    int n = 0;
    for (Future<?> j : jobs) { j.get(); if (++n % 1000 == 0) println("decompiled " + n); }
    ex.shutdown();
    try (PrintWriter w = new PrintWriter(new BufferedWriter(new FileWriter(D + "all_decomp_full.c")))) {
      for (int i = 0; i < fs.size(); i++) { w.println("//@@FUNC " + fa(fs.get(i))); w.print(out[i]); }
    }
    println("done");
  }
}
