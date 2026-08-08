#!/usr/bin/env python3
"""Print one Keep a Changelog version section without link definitions."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


def extract(text: str, version: str) -> str:
    heading = re.compile(rf"^## \[{re.escape(version)}\](?:\s+-.*)?\s*$", re.MULTILINE)
    matches = list(heading.finditer(text))
    if len(matches) != 1:
        raise ValueError(f"expected exactly one CHANGELOG section for {version}")

    start = matches[0].end()
    next_heading = re.search(r"^##\s+", text[start:], re.MULTILINE)
    end = start + next_heading.start() if next_heading else len(text)
    section = text[start:end]
    section = re.sub(r"^\[[^\]]+\]:\s+\S+\s*$", "", section, flags=re.MULTILINE)
    section = section.strip()
    if not section:
        raise ValueError(f"CHANGELOG section for {version} is empty")
    return section + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    parser.add_argument("--version", required=True)
    args = parser.parse_args()

    try:
        text = args.changelog.read_text(encoding="utf-8")
        sys.stdout.write(extract(text, args.version))
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
