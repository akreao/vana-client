#!/usr/bin/env python3
"""Play XI: Forever Offline with vana-client: their world without their game client.

XI: Forever Offline's launcher starts the retail client (Ashita, xiloader, Proton on Linux) when you press PLAY.
Their kit also runs the server side on its own, with no window and no game: `launcher_world.py run` starts the
database, applies the RULES / WORLD settings (rates.json), runs the game servers, the auction house simulator and
the co-op door. This script uses that, so their launcher stays your settings and tools app (open it while the world
is stopped) and vana-client is the game:

  1. Brings vana-client's own copy of their game folder up to date (their era DATs and DAT patches), copying only
     what changed since the last run. vana-client never plays from their folder itself.
  2. Starts their world and waits until it says it is running.
  3. Starts what their PLAY starts beside the world but the world does not: the local AI (Ollama, squad chat, the
     crowd and bot chat helper) when the RULES page has it on.
  4. Pins their server's TLS key for vana-client (a reinstall makes a new one), then opens vana-client on 127.0.0.1.
  5. When vana-client closes, stops all of it the normal way, so the database is saved and the next start is clean.

Only their own programs are run, unchanged, through their packaged launcher's `--run` (or python for a source
install). Nothing of theirs is copied into vana-client. Their launcher must be closed: the world refuses to start
while it holds the install.

Usage:
  tools/xifo-play.py --kit "/path/XI Forever Offline" [--client PATH] [--profile DIR] [--no-sync] [--no-ai]
                     [--world-only] [-- extra vana-client arguments]

  --kit        their install folder (the one with launcher_world.py and config.json). Env XIFO_KIT.
  --client     the vana-client program. Env VANA_CLIENT, else `vana-client` on PATH.
  --profile    where this XIFO setup of vana-client keeps its config.json, pins, log and mod settings, apart from
               your usual vana-client setup (FFXI_CONFIG). Default ~/.local/share/vana-client-xifo. Env XIFO_PROFILE.
  --game       vana-client's copy of their game folder. Default: the profile's config.json game_dir, else
               <profile>/game.
  --no-sync    do not refresh the game copy.
  --no-ai      do not start Ollama, squad chat or the chat helper.
  --world-only start the world (and the AI helpers) and keep it up until Ctrl+C, without opening vana-client.
  --allow-world-updates  leave their world's automatic updates as they are (see below).

Their world installs new releases by itself while it runs (world.json "auto_update", true when the file is not
there), restarting in the middle of play. A player install updates from the launcher instead, so on a kit with no
world.json this script writes one with "auto_update": false, unless --allow-world-updates is given.
"""
import argparse
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
import urllib.request

LOGIN_PORT = 54231
READY_SECONDS = 15 * 60          # the first start of a world can take a few minutes (their own limit is ten)
STOP_SECONDS = 4 * 60            # the world gives each game server 20 s to save, then stops the database
SYNC_STAMP = ".xifo-sync"        # in the game copy: when it was last brought up to date
SYNC_SKIP = ("FINAL FANTASY XI/USER",)   # the game writes each player's macros and settings here: keep ours


def say(text):
    print(f"[xifo-play] {text}", flush=True)


def fail(text):
    say(text)
    sys.exit(1)


def read_json(path, default=None):
    try:
        with open(path, encoding="utf-8-sig") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return default


def write_json(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path + ".part", "w", encoding="utf-8", newline="\n") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
    os.replace(path + ".part", path)


class Kit:
    """Their install: where things are, and how to run one of its scripts."""

    def __init__(self, path):
        self.path = os.path.abspath(path)
        if not os.path.isfile(os.path.join(self.path, "launcher_world.py")):
            fail(f"{self.path} is not an XI: Forever Offline install (no launcher_world.py there).")
        self.cfg = read_json(os.path.join(self.path, "config.json"))
        if not self.cfg:
            fail(f"{self.path} has no config.json yet: run their launcher's first setup once.")
        self.rates = read_json(os.path.join(self.path, "rates.json"), {}) or {}
        self.logs = os.path.join(self.path, "logs")
        exe = "XI Forever Offline.exe" if os.name == "nt" else "XI-Forever-Offline"
        self.exe = os.path.join(self.path, exe)

    def command(self, script, *args):
        """As their launcher runs its scripts: the packaged launcher's --run mode, or python for a source install."""
        if os.path.isfile(self.exe):
            return [self.exe, "--run", script] + list(args)
        return [sys.executable, "-u", script] + list(args)

    def server_dir(self):
        return self.cfg.get("server_dir") or os.path.join(self.path, "server")

    def game_dir(self):
        """Their SquareEnix folder (holding FINAL FANTASY XI and PlayOnlineViewer)."""
        ffxi = self.cfg.get("ffxi_client_dir") or ""
        return os.path.dirname(ffxi.rstrip("/\\")) if ffxi else ""


# ----- the game copy -----
def same_folder(source, target):
    return bool(source) and os.path.realpath(source) == os.path.realpath(target)


def sync_game(source, target):
    """Copies what changed in their game folder since the last run into ours (reflinks where the disk can).
    Files the game writes per player (USER: macros and settings) are copied only into a copy that has none, and
    after that left alone in both directions."""
    if not source or not os.path.isdir(os.path.join(source, "FINAL FANTASY XI")):
        fail(f"Their game folder was not found ({source or 'config.json has no ffxi_client_dir'}).")
    stamp = os.path.join(target, SYNC_STAMP)
    started = time.time()
    try:
        with open(stamp, encoding="utf-8") as handle:
            last_source = handle.read().strip()
        since = os.stat(stamp).st_mtime if last_source == source else 0   # another kit: everything again
    except OSError:
        since = 0
    has_user = any(os.path.isdir(os.path.join(target, rel)) for rel in SYNC_SKIP)
    if since == 0 and not (os.path.isdir(target) and os.listdir(target)):      # a new, empty copy
        say(f"Copying their game folder to {target} (once; later runs copy only what changed)...")
        os.makedirs(target, exist_ok=True)
        if shutil.which("cp") and os.name != "nt":
            subprocess.run(["cp", "-a", "--reflink=auto", os.path.join(source, "."), target], check=True)
        else:
            shutil.copytree(source, target, dirs_exist_ok=True)
    else:
        changed = []
        for root, dirs, files in os.walk(source):
            rel_root = os.path.relpath(root, source)
            dirs[:] = [d for d in dirs if os.path.normpath(os.path.join(rel_root, d)).replace("\\", "/") not in SYNC_SKIP
                       or not has_user]
            for name in files:
                path = os.path.join(root, name)
                try:
                    st = os.stat(path)
                except OSError:
                    continue
                # ctime too: an update or a DAT restore can put back a file with an old modification time
                if max(st.st_mtime, st.st_ctime) >= since:
                    changed.append(os.path.normpath(os.path.join(rel_root, name)))
        if changed:
            say(f"Their game files changed since the last run: copying {len(changed)} file(s)."
                if since else f"Bringing the game copy in line with {source}: {len(changed)} file(s).")
            copy_files(source, target, changed)
        else:
            say("The game copy is up to date.")
    with open(stamp, "w", encoding="utf-8") as handle:
        handle.write(f"{source}\n")
    os.utime(stamp, (started, started))


def copy_files(source, target, rels):
    if shutil.which("cp") and os.name != "nt":
        for i in range(0, len(rels), 400):
            subprocess.run(["cp", "-p", "--reflink=auto", "--parents", "--"] + rels[i:i + 400] + [target],
                           cwd=source, check=True)
        return
    for rel in rels:
        os.makedirs(os.path.dirname(os.path.join(target, rel)), exist_ok=True)
        shutil.copy2(os.path.join(source, rel), os.path.join(target, rel))


# ----- vana-client's setup for this world -----
def prepare_profile(profile, game):
    """The profile's config.json, pointed at 127.0.0.1 and the game copy; everything else in it is the client's."""
    path = os.path.join(profile, "config.json")
    cfg = read_json(path, {}) or {}
    if not game:
        game = cfg.get("game_dir") or os.path.join(profile, "game")
    want = {"server": "127.0.0.1", "game_dir": os.path.abspath(game), "installed": True}
    if any(cfg.get(k) != v for k, v in want.items()):
        cfg.update(want)
        write_json(path, cfg)
        say(f"vana-client's XIFO settings: {path} (server 127.0.0.1, game {want['game_dir']}).")
    return path, want["game_dir"]


def server_key(server_dir):
    """SHA-256 of their login certificate's public key (what vana-client pins), or None without openssl."""
    cert = os.path.join(server_dir, "login.cert")
    if not os.path.isfile(cert) or not shutil.which("openssl"):
        return None
    try:
        pem = subprocess.run(["openssl", "x509", "-in", cert, "-pubkey", "-noout"], capture_output=True, check=True).stdout
        der = subprocess.run(["openssl", "pkey", "-pubin", "-outform", "DER"], input=pem, capture_output=True,
                             check=True).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    return hashlib.sha256(der).hexdigest()


def pin_server(client, env, profile, key):
    """Pins their key for the login port. After a reinstall (a new key) the old pins go first, so vana-client
    connects instead of refusing a changed certificate."""
    if key is None:
        say("openssl not found: vana-client pins the server's key on its first connection, as usual.")
        return
    known = read_json(os.path.join(profile, "known_servers.json"), {}) or {}
    pinned = (known.get(f"127.0.0.1:{LOGIN_PORT}") or {}).get("sha256")
    if pinned == key:
        return
    if pinned:
        say("Their server has a new TLS key (a reinstall?): vana-client forgets the old one.")
        for extra in ([], ["--port", str(LOGIN_PORT)]):
            subprocess.run([client, "trust-server", "127.0.0.1"] + extra, env=env, check=False, stdout=subprocess.DEVNULL)
    done = subprocess.run([client, "trust-server", "127.0.0.1", "--port", str(LOGIN_PORT), "--fingerprint", key],
                          env=env, stdout=subprocess.DEVNULL)
    say("Pinned their server's TLS key for vana-client." if done.returncode == 0
        else "vana-client could not pin their server's key; it pins it on its first connection instead.")


# ----- their world -----
class World:
    def __init__(self, kit):
        self.kit = kit
        self.proc = None
        self.helpers = {}
        self.ollama = None
        self.log_start = 0

    def keep_updates_off(self, allowed):
        path = os.path.join(self.kit.path, "world.json")
        if allowed or os.path.exists(path):
            return
        write_json(path, {"auto_update": False})
        say("Wrote world.json with automatic world updates off (update from their launcher as before).")

    def status(self):
        return read_json(os.path.join(self.kit.logs, "world-status.json"), {}) or {}

    def world_log_since_start(self):
        try:
            with open(os.path.join(self.kit.logs, "world.log"), encoding="utf-8", errors="replace") as handle:
                handle.seek(self.log_start)
                return handle.read()
        except OSError:
            return ""

    def start(self, stopping=lambda: False):
        os.makedirs(self.kit.logs, exist_ok=True)
        try:
            self.log_start = os.path.getsize(os.path.join(self.kit.logs, "world.log"))
        except OSError:
            self.log_start = 0
        say("Starting their world (database, rules, game servers)...")
        began = time.time()
        self.proc = subprocess.Popen(self.kit.command("launcher_world.py", "run"), cwd=self.kit.path,
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **own_group())
        shown = ""
        while time.time() - began < READY_SECONDS:
            if stopping():
                self.stop()
                fail("Stopped while their world was starting.")
            if self.proc.poll() is not None:
                tail = "\n".join(self.world_log_since_start().strip().splitlines()[-12:])
                fail(f"Their world stopped while starting (exit code {self.proc.returncode}). Is their launcher "
                     f"still open? Its log (logs/world.log):\n{tail}")
            st = self.status()
            # (their packaged launcher is a one-file build: the world runs in a child of the process we started and
            #  writes the child's pid, so the start time tells this start's status from an older one)
            if st.get("state") == "running" and (st.get("since") or 0) >= int(began) - 1:
                say(f"Their world is running ({int(time.time() - began)} seconds).")
                return
            lines = self.world_log_since_start().strip().splitlines()
            if lines and lines[-1] != shown:
                shown = lines[-1]
                say("world: " + shown.split(" ", 2)[-1])
            time.sleep(1)
        self.stop()
        fail("Their world did not get ready in time; see logs/world.log in their install.")

    def alive(self):
        return self.proc is not None and self.proc.poll() is None

    def start_ai(self):
        """What their PLAY starts beside the servers and the world leaves out: the local AI (launcher_gui.pyw)."""
        rates = self.kit.rates
        squad = bool(rates.get("squad_chat", True))
        chat_ai = bool(rates.get("crowds_chat_ai", True))
        crowds_ai = chat_ai and rates.get("crowds", True) and rates.get("crowds_chat", True)
        bots_ai = chat_ai and rates.get("bots", True)
        cloud = "Bot chat through Google's Gemma started." in self.world_log_since_start()  # the world ran it already
        local_chat = (crowds_ai or bots_ai) and not cloud
        if not (squad or local_chat):
            say("Local AI is off on their RULES page (squad chat, chat AI): nothing more to start.")
            return
        self.start_ollama()
        if squad:
            self.helper("squad chat", "squad-chat", "squad_chat.py", "squad-chat.log")
        if local_chat:
            self.helper("chat AI helper", "crowds", "crowd_chat_ai.py", "crowd-chat-ai.log",
                        *([] if crowds_ai else ["--bots-only"]))

    def start_ollama(self):
        cfg = read_json(os.path.join(self.kit.path, "squad-chat", "config.json"), {}) or {}
        url = (cfg.get("ollama_url") or "http://127.0.0.1:11434").rstrip("/") + "/api/tags"

        def running():
            try:
                with urllib.request.urlopen(url, timeout=2):
                    return True
            except Exception:
                return False

        if running():
            return
        exe = shutil.which("ollama") or next((p for p in (os.path.join(self.kit.path, "ollama", "bin", "ollama"),)
                                             if os.path.isfile(p)), None)
        if not exe:
            say("Ollama was not found: AI chat stays off (their bots use their short lines).")
            return
        log = open(os.path.join(self.kit.logs, "ollama.log"), "ab")
        self.ollama = subprocess.Popen([exe, "serve"], stdout=log, stderr=subprocess.STDOUT,
                                       **own_group())
        for _ in range(20):
            if running():
                break
            time.sleep(0.5)
        say("Ollama started.")

    def helper(self, name, folder, script, log_name, *args):
        where = os.path.join(self.kit.path, folder)
        if not os.path.isfile(os.path.join(where, script)):
            return
        log = open(os.path.join(self.kit.logs, log_name), "wb")
        self.helpers[name] = subprocess.Popen(self.kit.command(os.path.join(where, script), *args), cwd=where,
                                              stdout=log, stderr=subprocess.STDOUT,
                                              **own_group())
        say(f"Started {name}.")

    def stop(self):
        for name, proc in list(self.helpers.items()) + ([("Ollama", self.ollama)] if self.ollama else []):
            end(proc, 10)
        self.helpers.clear()
        self.ollama = None
        if not self.alive():
            return
        say("Stopping their world (it saves and stops the database)...")
        try:
            # (Windows: Ctrl+Break, which their world takes as a stop like SIGTERM; it reaches the process group
            #  started with CREATE_NEW_PROCESS_GROUP, so the packaged launcher's child gets it too)
            self.proc.send_signal(signal.CTRL_BREAK_EVENT if os.name == "nt" else signal.SIGTERM)
            self.proc.wait(timeout=STOP_SECONDS)
            say("Their world stopped.")
        except subprocess.TimeoutExpired:
            say("Their world did not stop in time; ending it.")
            end(self.proc, 5)


def own_group():
    """Popen arguments that start a program in its own process group, so it can be stopped with its children."""
    if os.name == "nt":
        return {"creationflags": subprocess.CREATE_NEW_PROCESS_GROUP}
    return {"start_new_session": True}


def end(proc, seconds):
    if proc is None or proc.poll() is not None:
        return
    try:
        if os.name != "nt":
            os.killpg(proc.pid, signal.SIGTERM)
        else:
            proc.terminate()
        proc.wait(timeout=seconds)
    except (OSError, subprocess.TimeoutExpired):
        try:
            if os.name != "nt":
                os.killpg(proc.pid, signal.SIGKILL)
            else:
                proc.kill()
        except OSError:
            pass


def main(argv):
    parser = argparse.ArgumentParser(description="Play XI: Forever Offline with vana-client (see the top of this file).")
    parser.add_argument("--kit", default=os.environ.get("XIFO_KIT"))
    parser.add_argument("--client", default=os.environ.get("VANA_CLIENT") or shutil.which("vana-client"))
    parser.add_argument("--profile", default=os.environ.get("XIFO_PROFILE")
                        or os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"),
                                        "vana-client-xifo"))
    parser.add_argument("--game")
    parser.add_argument("--no-sync", action="store_true")
    parser.add_argument("--no-ai", action="store_true")
    parser.add_argument("--world-only", action="store_true")
    parser.add_argument("--allow-world-updates", action="store_true")
    parser.add_argument("client_args", nargs="*")
    args = parser.parse_args(argv)
    if not args.kit:
        fail("Where is XI: Forever Offline installed? Give --kit (or set XIFO_KIT).")
    kit = Kit(args.kit)
    client = None
    if not args.world_only:
        client = args.client
        if not client or not (os.path.isfile(client) or shutil.which(client)):
            fail("vana-client was not found: give --client (or set VANA_CLIENT).")
    profile = os.path.abspath(args.profile)
    config, game = prepare_profile(profile, args.game)
    if same_folder(kit.game_dir(), game):
        fail("vana-client's game folder is their game folder: give it its own copy (--game).")
    if not args.no_sync:
        sync_game(kit.game_dir(), game)

    world = World(kit)
    stopping = []

    def asked_to_stop(_number, _frame):
        stopping.append(True)

    for name in ("SIGINT", "SIGTERM", "SIGHUP"):
        if hasattr(signal, name):
            signal.signal(getattr(signal, name), asked_to_stop)

    code = 0
    try:
        world.keep_updates_off(args.allow_world_updates)
        world.start(lambda: bool(stopping))
        if not args.no_ai:
            world.start_ai()
        if args.world_only:
            say("The world is up. Ctrl+C stops it.")
            while not stopping and world.alive():
                time.sleep(1)
        else:
            env = dict(os.environ, FFXI_CONFIG=config)
            pin_server(client, env, profile, server_key(kit.server_dir()))
            say("Opening vana-client. Closing it stops the world.")
            game_proc = subprocess.Popen([client] + args.client_args, env=env, **own_group())
            while game_proc.poll() is None:
                if stopping:
                    end(game_proc, 20)
                    break
                if not world.alive():
                    say("Their world stopped by itself (see logs/world.log); vana-client stays open.")
                    game_proc.wait()
                    code = 1
                    break
                time.sleep(1)
            if code == 0 and not stopping and game_proc.returncode:
                say(f"vana-client ended with exit code {game_proc.returncode}.")
                code = game_proc.returncode
        if not world.alive() and not stopping:
            code = 1
    finally:
        world.stop()
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
