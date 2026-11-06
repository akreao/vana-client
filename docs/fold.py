#!/usr/bin/env python3
"""Fold the per-change doc fragments into their shared files.

- docs/modlog/*.md     -> appended to the end of MODLOG.md (each starts with a '## Title (date)' heading)
- docs/modern-ui/*.md  -> appended to the State section of docs/modern-ui.md (each is '- ' bullets),
                          above its closing '- Not yet:' lines

Fragments are folded oldest file name first and then deleted. Run on custom (the fork's default branch), not on a feature branch.
See docs/modlog/README.md and docs/modern-ui/README.md.
"""

import argparse
import re
import sys
from pathlib import Path

DOCS = Path(__file__).resolve().parent
ROOT = DOCS.parent
NAME = re.compile(r"^\d{4}-\d{2}-\d{2}-[A-Za-z0-9._-]+\.md$")


def append_end(log, added):
    return log.rstrip("\n") + "\n" + added


def append_state(doc, added):
    """Insert before the trailing '- Not yet:' bullets of the last section (State)."""
    lines = doc.rstrip("\n").split("\n")
    i = len(lines)
    while i > 0 and lines[i - 1].startswith("- Not yet:"):
        i -= 1
    head, tail = lines[:i], lines[i:]
    body = added.strip("\n").split("\n")
    return "\n".join(head + body + tail) + "\n"


TARGETS = [
    # folder, target file, what a fragment must start with, how to join
    ("modlog", ROOT / "MODLOG.md", "## ", lambda t: "\n" + t + "\n", append_end),
    ("modern-ui", DOCS / "modern-ui.md", "- ", lambda t: t + "\n", append_state),
]


def fragments(folder):
    return sorted(p for p in (DOCS / folder).glob("*.md") if p.name != "README.md")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="only validate the fragments")
    mode.add_argument("--print", action="store_true", help="print what would be added, change nothing")
    args = ap.parse_args()

    problems, plans = [], []
    for folder, target, start, wrap, join in TARGETS:
        paths = fragments(folder)
        for p in paths:
            if not NAME.match(p.name):
                problems.append(f"{folder}/{p.name}: name should be YYYY-MM-DD-short-name.md")
            if not p.read_text(encoding="utf-8").strip().startswith(start):
                problems.append(f"{folder}/{p.name}: should start with '{start}'")
        plans.append((folder, target, paths, wrap, join))
    if problems:
        print("\n".join(problems), file=sys.stderr)
        return 1

    for folder, target, paths, wrap, join in plans:
        if args.check:
            print(f"{folder}: {len(paths)} fragment(s) OK")
            continue
        if not paths:
            print(f"{folder}: nothing to fold")
            continue
        added = "".join(wrap(p.read_text(encoding="utf-8").strip()) for p in paths)
        if args.print:
            print(f"--- {folder} -> {target.relative_to(ROOT)}")
            sys.stdout.write(added)
            continue
        target.write_text(join(target.read_text(encoding="utf-8"), added), encoding="utf-8")
        for p in paths:
            p.unlink()
        print(f"{folder}: folded {len(paths)} fragment(s) into {target.relative_to(ROOT)}")
        for p in paths:
            print(f"  {p.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
