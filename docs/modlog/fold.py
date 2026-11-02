#!/usr/bin/env python3
"""Fold docs/modlog/*.md fragments into MODLOG.md (see docs/modlog/README.md)."""

import argparse
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODLOG = HERE.parent.parent / "MODLOG.md"
NAME = re.compile(r"^\d{4}-\d{2}-\d{2}-[A-Za-z0-9._-]+\.md$")


def fragments():
    return sorted(p for p in HERE.glob("*.md") if p.name != "README.md")


def check(paths):
    problems = []
    for p in paths:
        text = p.read_text(encoding="utf-8").strip()
        if not NAME.match(p.name):
            problems.append(f"{p.name}: name should be YYYY-MM-DD-short-name.md")
        if not text.startswith("## "):
            problems.append(f"{p.name}: should start with a '## Title (YYYY-MM-DD)' heading")
    return problems


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="only validate the fragments")
    mode.add_argument("--print", action="store_true", help="print what would be appended, change nothing")
    args = ap.parse_args()

    paths = fragments()
    problems = check(paths)
    if problems:
        print("\n".join(problems), file=sys.stderr)
        return 1
    if args.check:
        print(f"{len(paths)} fragment(s) OK")
        return 0
    if not paths:
        print("no fragments to fold")
        return 0

    added = "".join("\n" + p.read_text(encoding="utf-8").strip() + "\n" for p in paths)
    if args.print:
        sys.stdout.write(added)
        return 0

    log = MODLOG.read_text(encoding="utf-8").rstrip("\n") + "\n"
    MODLOG.write_text(log + added, encoding="utf-8")
    for p in paths:
        p.unlink()
    print(f"folded {len(paths)} fragment(s) into MODLOG.md:")
    for p in paths:
        print(f"  {p.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
