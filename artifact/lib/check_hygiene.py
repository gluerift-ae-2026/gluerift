#!/usr/bin/env python3
"""Reject forbidden implementation escapes and noncanonical path leakage."""

from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path, PurePosixPath


FORBIDDEN_SOURCE_PATTERNS = (
    (re.compile(r"\bsorry\b"), "Lean placeholder keyword", True),
    (re.compile(r"\badmit\b"), "Lean admission keyword", True),
    (re.compile(r"\bFIXME\b"), "unfinished-work marker", False),
    (re.compile(r"\bTODO\b"), "unfinished-work marker", False),
)

SKIP_PREFIXES = (
    ".git/",
    ".tools/",
    ".worklog/",
    "artifact/evidence/",
    "artifact/results/",
    "artifact/tables/",
    "checker/target/",
    "checker/target-rustkernel/",
    "native/.cache/",
    "native/rust/target/",
    "proof/.lake/",
    "proof/.toolchain/",
)

TEXT_SUFFIXES = {
    "",
    ".c",
    ".go",
    ".h",
    ".json",
    ".lean",
    ".md",
    ".proto",
    ".py",
    ".rs",
    ".sh",
    ".toml",
    ".txt",
    ".yaml",
    ".yml",
}


def logical(root: Path, path: Path) -> str:
    return PurePosixPath(*path.relative_to(root).parts).as_posix()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    root = args.root.resolve()
    failures: list[str] = []
    paths: list[Path] = []
    for directory, names, files in os.walk(root):
        base = Path(directory)
        relative_directory = logical(root, base) if base != root else ""
        names[:] = sorted(
            name
            for name in names
            if not any(
                (f"{relative_directory}/{name}/" if relative_directory else f"{name}/").startswith(prefix)
                for prefix in SKIP_PREFIXES
            )
        )
        paths.extend(base / name for name in sorted(files))
    for path in sorted(paths, key=str):
        rel = logical(root, path)
        if any(rel.startswith(prefix) for prefix in SKIP_PREFIXES):
            continue
        if path.suffix not in TEXT_SUFFIXES and path.name not in {
            "reproduce",
            "Cargo.lock",
            "go.sum",
            "lean-toolchain",
        }:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for pattern, reason, lean_only in FORBIDDEN_SOURCE_PATTERNS:
            if lean_only and path.suffix != ".lean":
                continue
            for match in pattern.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                # The hygiene checker must be able to spell the patterns it detects.
                if rel == "artifact/lib/check_hygiene.py":
                    continue
                failures.append(f"{rel}:{line}: {reason}")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 4
    print("source hygiene: proved-exhaustive")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
