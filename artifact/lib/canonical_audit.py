#!/usr/bin/env python3
"""Audit canonical evidence for byte form, typed absence, and path hygiene."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path, PurePosixPath
from typing import Any

from jcs import canonical_bytes, load_json


class CanonicalAuditError(ValueError):
    pass


FORBIDDEN_AMBIENT_KEYS = {
    "created_at",
    "creation_time",
    "host",
    "host_name",
    "hostname",
    "timestamp",
    "wall_clock_duration",
}

ABSOLUTE_ALLOWED_KEYS = {
    "compiler_absolute_path",
    "image_internal_path",
    "linker_absolute_path",
    "tool_absolute_path",
}

LOGICAL_PATH_KEYS = {
    "context_logical_path",
    "logical_path",
    "output_logical_path",
    "policy_logical_path",
    "request_logical_path",
    "scope_logical_path",
    "stdin_or_fixture_logical_path",
    "transformation_base_context_logical_path",
    "working_directory",
}

LOGICAL_PATH_MAP_KEYS = {
    "declared_input_hashes",
    "lockfile_hashes",
    "source_file_hashes",
}

HOST_PATH_PATTERNS = (
    re.compile(r"(?:^|[= :])/Users/"),
    re.compile(r"(?:^|[= :])/private/"),
    re.compile(r"(?:^|[= :])/tmp(?:/|$)"),
    re.compile(r"(?:^|[= :])/var/folders/"),
    re.compile(r"(?:^|[= :])/home/"),
    re.compile(r"^[A-Za-z]:\\"),
)


def audit_logical_path(value: str, path: str) -> None:
    if value in {".", "not-applicable"}:
        return
    pure = PurePosixPath(value)
    if pure.is_absolute() or not pure.parts or ".." in pure.parts:
        raise CanonicalAuditError(f"{path}: logical path is not repository-relative")


def audit_value(value: Any, path: str = "$", key: str | None = None) -> None:
    if value is None:
        raise CanonicalAuditError(f"{path}: null is forbidden; use a typed absence value")
    if isinstance(value, float):
        raise CanonicalAuditError(f"{path}: floating point is forbidden")
    if isinstance(value, (bool, int)):
        return
    if isinstance(value, str):
        if key in LOGICAL_PATH_KEYS or (key is not None and key.endswith("_logical_path")):
            audit_logical_path(value, path)
        if any(pattern.search(value) for pattern in HOST_PATH_PATTERNS) and key not in ABSOLUTE_ALLOWED_KEYS:
            raise CanonicalAuditError(f"{path}: host-specific absolute path leaked into evidence")
        return
    if isinstance(value, list):
        for index, item in enumerate(value):
            audit_value(item, f"{path}[{index}]", key)
        return
    if isinstance(value, dict):
        forbidden = sorted(set(value) & FORBIDDEN_AMBIENT_KEYS)
        if forbidden:
            raise CanonicalAuditError(f"{path}: ambient telemetry keys are forbidden: {forbidden}")
        if key in LOGICAL_PATH_MAP_KEYS:
            for logical in value:
                audit_logical_path(logical, f"{path}.<key>")
        if key == "canonical_tool_paths":
            for image_path in value:
                if not image_path.startswith("/opt/gluerift/"):
                    raise CanonicalAuditError(
                        f"{path}: image tool path is outside the canonical /opt/gluerift namespace"
                    )
        for child_key, child in value.items():
            audit_value(child, f"{path}.{child_key}", child_key)
        return
    raise CanonicalAuditError(f"{path}: value is not JSON")


def audit_file(path: Path, require_canonical_bytes: bool) -> None:
    value = load_json(path)
    audit_value(value)
    if require_canonical_bytes:
        expected = canonical_bytes(value)
        if path.read_bytes() != expected:
            raise CanonicalAuditError(f"{path}: bytes are not the exact RFC 8785 serialization")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", type=Path, nargs="+")
    parser.add_argument("--require-canonical-bytes", action="store_true")
    args = parser.parse_args()
    files: list[Path] = []
    for path in args.paths:
        if path.is_dir():
            files.extend(sorted(path.rglob("*.json")))
        else:
            files.append(path)
    if not files:
        raise CanonicalAuditError("no canonical JSON files selected")
    for path in files:
        audit_file(path, args.require_canonical_bytes)
    print(f"canonical evidence audit: proved-exhaustive ({len(files)} files)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, CanonicalAuditError) as error:
        print(f"canonical evidence error: {error}", file=sys.stderr)
        raise SystemExit(4)
