#!/usr/bin/env python3
"""A new retail FFXiMain.dll build, from the DLL to the packet diff, in one run (steps 1 and 2 of "A new retail
build" in README.md, then packets.py).

    python3 newbuild.py VERSION [--old VERSION] [--dll FILE | --fetch] [--sum HEX] [--queue FILE] [--force]
    python3 newbuild.py watch [--repo OWNER/NAME] [--post] [--queue FILE]

Every build has a folder BUILDS/<version>/ (BUILDS = --builds or $FFXI_BUILDS). For VERSION it:
  1. takes the DLL: --dll FILE, --fetch (the patch server's current FFXiMain.dll through `vana-client update --file`,
     which must be VERSION), or the FFXiMain.dll already in the folder; --sum checks its 0x26 checksum
     (net/lobby.rs ffximain_checksum, as assets/known-builds.txt lists it);
  2. unpacks it (unpack_pol1.py) and exports the index (ri.sh: Ghidra import and analysis on the first run);
  3. compares it with the build before (--old, else the newest older folder): remap.py -> <version>/remap/;
  4. packets.py diff -> <version>/packet-diff/ (report.md, summary.md, queue-rows.md); --queue appends the rows.
Steps already done are skipped while their outputs are newer than their inputs (--force redoes the new build's;
an older build's existing index is used as it is). One run at a
time (a lock in BUILDS): Ghidra is heavy.

`watch` is the local job for the known-builds workflow's PRs: for each open "Known build <version> (FFXiMain.dll
<sum>)" PR not handled yet it runs the above with --fetch --sum, and with --post comments summary.md on the PR (ids,
sizes, addresses and our file names only). It reports; it changes no code.

Environment: FFXI_BUILDS, VANA_CLIENT (the client binary for --fetch; default: target/release/vana-client of this
checkout, else `cargo run --release -p vana-client`), GHIDRA_HOME (ri.sh). Everything written is generated from the
DLL and stays local.
"""
import sys, os, re, json, shutil, argparse, subprocess, fcntl, time

HERE = os.path.dirname(os.path.abspath(__file__))
VERSION_RE = re.compile(r'^\d{8}_\d+$')


def log(*a):
    print('[newbuild %s]' % time.strftime('%H:%M:%S'), *a, flush=True)


def checksum(path):
    """FUN_100185a0 over the packed DLL: signed bytes shifted left by 4 x (i mod 20) (shift mod 32), summed."""
    s = 0
    data = open(path, 'rb').read()
    for k in range(20):
        sh = (k * 4) & 31
        part = data[k::20]
        neg = sum(1 for b in part if b >= 0x80)
        s += (sum(part) - 256 * neg) << sh
    return s & 0xffffffff


def newer(out, *inputs):
    """`out` exists and is newer than every existing input."""
    if not os.path.exists(out):
        return False
    t = os.path.getmtime(out)
    return all(not os.path.exists(i) or os.path.getmtime(i) <= t for i in inputs)


def work_dir(d):
    """The dll-index work dir of a build folder: the folder, or its base/ export when that is where the index is."""
    for w in (os.path.join(d, 'base'), d):
        if os.path.exists(os.path.join(w, 'sig_insns.tsv')) and os.path.exists(os.path.join(w, 'all_decomp_full.c')):
            return w
    return d


def ver_key(v):
    a, b = v.split('_')
    return int(a), int(b)


def client_cmd(src):
    if os.environ.get('VANA_CLIENT'):
        return [os.environ['VANA_CLIENT']]
    exe = os.path.join(src, 'target', 'release', 'vana-client')
    if os.path.exists(exe):
        return [exe]
    return ['cargo', 'run', '-q', '--release', '-p', 'vana-client', '--manifest-path',
            os.path.join(src, 'Cargo.toml'), '--']


def run(cmd, **kw):
    log('$', ' '.join(cmd))
    subprocess.run(cmd, check=True, **kw)


def checkout():
    p = subprocess.run(['git', '-C', HERE, 'rev-parse', '--show-toplevel'], capture_output=True, text=True)
    return p.stdout.strip() or None


def build(a, builds):
    ver = a.version
    if not VERSION_RE.match(ver):
        raise SystemExit('version %r is not DATE_N' % ver)
    nd = os.path.join(builds, ver)
    os.makedirs(nd, exist_ok=True)
    packed = os.path.join(nd, 'FFXiMain.dll')
    src = checkout()

    # 1. the DLL
    if a.dll:
        if os.path.abspath(a.dll) != os.path.abspath(packed):
            shutil.copyfile(a.dll, packed)
    elif a.fetch:
        tmp = os.path.join(nd, 'fetch')
        os.makedirs(tmp, exist_ok=True)
        p = subprocess.run(client_cmd(src or HERE) + ['update', '--file', 'FFXiMain.dll', '--to', tmp],
                           capture_output=True, text=True)
        sys.stdout.write(p.stdout)
        if p.returncode != 0:
            sys.stderr.write(p.stderr)
            raise SystemExit('fetching FFXiMain.dll failed')
        m = re.search(r'wrote (.+) \(version (\S+)\)', p.stdout)
        if not m:
            raise SystemExit('unexpected `vana-client update --file` output')
        if m.group(2) != ver:
            raise SystemExit('the patch server is on %s now, not %s' % (m.group(2), ver))
        os.replace(m.group(1), packed)
        shutil.rmtree(tmp, ignore_errors=True)
    if not os.path.exists(packed):
        raise SystemExit('no %s: pass --dll FILE or --fetch' % packed)
    sum_ = checksum(packed)
    log('FFXiMain.dll %s checksum %08X' % (ver, sum_))
    if a.sum and int(a.sum, 16) != sum_:
        raise SystemExit('checksum %08X, expected %s' % (sum_, a.sum.upper()))

    # 2. unpack and export
    unpacked = os.path.join(nd, 'FFXiMain.unpacked.dll')
    if a.force or not newer(unpacked, packed):
        run([sys.executable, os.path.join(HERE, 'unpack_pol1.py'), packed, unpacked])
    nw = work_dir(nd)
    if a.force or not newer(os.path.join(nw, 'sig_insns.tsv'), unpacked):
        nw = nd
        run(['sh', os.path.join(HERE, 'ri.sh')], env=dict(os.environ, FFXI_RE_DIR=nd))

    # 3. the build before, and the comparison
    old = a.old
    if not old:
        older = sorted((v for v in os.listdir(builds) if VERSION_RE.match(v) and ver_key(v) < ver_key(ver) and
                        os.path.exists(os.path.join(builds, v, 'FFXiMain.unpacked.dll'))), key=ver_key)
        if not older:
            raise SystemExit('no older build folder in %s to compare with (pass --old)' % builds)
        old = older[-1]
    od = os.path.join(builds, old)
    ow = work_dir(od)
    if not os.path.exists(os.path.join(ow, 'sig_insns.tsv')):
        if not os.path.exists(os.path.join(ow, 'FFXiMain.unpacked.dll')):
            raise SystemExit('%s: no index and no unpacked DLL' % ow)
        run(['sh', os.path.join(HERE, 'ri.sh')], env=dict(os.environ, FFXI_RE_DIR=ow))
    log('comparing %s with %s' % (ver, old))
    if a.force or not newer(os.path.join(nw, 'remap', 'summary.txt'), os.path.join(nw, 'sig_insns.tsv'),
                            os.path.join(ow, 'sig_insns.tsv')):
        run([sys.executable, os.path.join(HERE, 'remap.py'), ow, nw])

    # 4. packets
    out = os.path.join(nd, 'packet-diff')
    cmd = [sys.executable, os.path.join(HERE, 'packets.py'), 'diff', ow, nw, '--out', out]
    if src:
        cmd += ['--src', src]
    if a.queue:
        cmd += ['--queue', a.queue]
    run(cmd)
    with open(os.path.join(out, 'build.json'), 'w') as f:
        json.dump({'version': ver, 'checksum': '%08X' % sum_, 'old': old, 'old_dir': ow, 'new_dir': nw}, f)
    return out


def watch(a, builds):
    src = checkout()
    repo = a.repo or subprocess.run(['gh', 'repo', 'view', '--json', 'nameWithOwner', '-q', '.nameWithOwner'],
                                    cwd=src or HERE, capture_output=True, text=True, check=True).stdout.strip()
    prs = json.loads(subprocess.run(['gh', 'pr', 'list', '--repo', repo, '--state', 'open', '--search',
                                     'in:title "Known build"', '--json', 'number,title,url'],
                                    capture_output=True, text=True, check=True).stdout)
    rc = 0
    for pr in prs:
        m = re.match(r'^Known build (\d{8}_\d+) \(FFXiMain\.dll ([0-9A-Fa-f]{8})\)', pr['title'])
        if not m:
            continue
        ver, sum_ = m.groups()
        mark = os.path.join(builds, ver, 'packet-diff', 'pr-%d-%s.done' % (pr['number'], sum_.upper()))
        if os.path.exists(mark):
            continue
        log('PR #%d: %s' % (pr['number'], pr['title']))
        args = argparse.Namespace(version=ver, old=None, dll=None, fetch=True, sum=sum_, queue=a.queue,
                                  force=False)
        if os.path.exists(os.path.join(builds, ver, 'FFXiMain.dll')) and \
                checksum(os.path.join(builds, ver, 'FFXiMain.dll')) == int(sum_, 16):
            args.fetch = False     # already here
        try:
            out = build(args, builds)
        except (SystemExit, subprocess.CalledProcessError) as e:
            log('PR #%d failed: %s' % (pr['number'], e))
            rc = 1
            continue
        if a.post:
            run(['gh', 'pr', 'comment', str(pr['number']), '--repo', repo, '--body-file',
                 os.path.join(out, 'summary.md')])
        with open(mark, 'w') as f:
            f.write(time.strftime('%Y-%m-%d %H:%M:%S\n'))
    return rc


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('--builds', default=os.environ.get('FFXI_BUILDS'), help='the build folders ($FFXI_BUILDS)')
    ap.add_argument('--queue', help='modlog-gaps queue file to append new rows to')
    if len(sys.argv) > 1 and sys.argv[1] == 'watch':
        ap.add_argument('cmd')
        ap.add_argument('--repo')
        ap.add_argument('--post', action='store_true', help='comment the summary on the PR')
    else:
        ap.add_argument('version')
        ap.add_argument('--old')
        g = ap.add_mutually_exclusive_group()
        g.add_argument('--dll')
        g.add_argument('--fetch', action='store_true')
        ap.add_argument('--sum')
        ap.add_argument('--force', action='store_true')
    a = ap.parse_args()
    if not a.builds:
        raise SystemExit('pass --builds DIR or set FFXI_BUILDS')
    builds = os.path.abspath(a.builds)
    os.makedirs(builds, exist_ok=True)
    with open(os.path.join(builds, '.newbuild.lock'), 'w') as lk:
        try:
            fcntl.flock(lk, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit('another newbuild.py run holds %s/.newbuild.lock' % builds)
        if getattr(a, 'cmd', None) == 'watch':
            sys.exit(watch(a, builds))
        out = build(a, builds)
        log('done: %s' % out)


if __name__ == '__main__':
    main()
