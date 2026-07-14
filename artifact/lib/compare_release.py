#!/usr/bin/env python3
"""Byte-compare a staged release graph with the checked-in canonical release."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path, PurePosixPath

from jcs import load_json


class CompareError(ValueError):
    pass


def _resolve(root: Path, logical: str) -> Path:
    pure = PurePosixPath(logical)
    if pure.is_absolute() or not pure.parts or ".." in pure.parts:
        raise CompareError(f"invalid logical release path: {logical}")
    path = root.joinpath(*pure.parts)
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise CompareError(f"release path escapes root: {logical}") from error
    return path


def _manifest_paths(document: dict) -> list[str]:
    paths = [
        entry["logical_path"]
        for layer in document["layers"]
        for entry in layer["entries"]
    ]
    paths.append("artifact/reproduction-manifest.json")
    if len(paths) != len(set(paths)):
        raise CompareError("release graph contains duplicate paths")
    return sorted(paths)


def compare(expected_root: Path, actual_root: Path) -> None:
    expected_manifest_path = expected_root / "artifact/reproduction-manifest.json"
    actual_manifest_path = actual_root / "artifact/reproduction-manifest.json"
    expected_manifest = load_json(expected_manifest_path)
    actual_manifest = load_json(actual_manifest_path)
    expected_paths = _manifest_paths(expected_manifest)
    actual_paths = _manifest_paths(actual_manifest)
    if expected_paths != actual_paths:
        missing = sorted(set(expected_paths) - set(actual_paths))
        extra = sorted(set(actual_paths) - set(expected_paths))
        raise CompareError(f"release path set differs; missing={missing}, extra={extra}")
    for logical in expected_paths:
        expected = _resolve(expected_root, logical)
        actual = _resolve(actual_root, logical)
        if not expected.is_file() or not actual.is_file():
            raise CompareError(f"release artifact missing: {logical}")
        if expected.read_bytes() != actual.read_bytes():
            raise CompareError(f"release artifact is not byte-identical: {logical}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--expected-root", type=Path, required=True)
    parser.add_argument("--actual-root", type=Path, required=True)
    args = parser.parse_args()
    compare(args.expected_root.resolve(), args.actual_root.resolve())
    print("release byte comparison: proved-exhaustive")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, CompareError) as error:
        print(f"release comparison error: {error}", file=sys.stderr)
        raise SystemExit(4)
