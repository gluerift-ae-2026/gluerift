#!/usr/bin/env python3
"""Build and verify the acyclic GlueRift primary source-input manifest."""

from __future__ import annotations

import argparse
import os
import stat
import sys
from pathlib import Path, PurePosixPath

from jcs import canonical_bytes, canonical_sha256, load_json, sha256_bytes, write_canonical


CONTRACT = "ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md"
APPROVED_CONTRACT_SHA256 = (
    "1b0ebee64fcb482f87e1d37bece9a5ae2fc44bac7121607f31a531ea9dcf9fc7"
)

EXCLUDED_PREFIXES = (
    ".git/",
    ".tools/",
    ".worklog/",
    "artifact/evidence/",
    "artifact/.staging/",
    "artifact/results/",
    "artifact/tables/",
    "checker/target/",
    "checker/target-rustkernel/",
    "native/.cache/",
    "native/rust/target/",
    "output/",
    "paper/generated/",
    "tmp/",
    "proof/.lake/",
    "proof/.toolchain/",
)

EXCLUDED_BASENAMES = {
    ".DS_Store",
    "ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.md",
    "ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1.md",
}

EXCLUDED_EXACT = {
    "artifact/claims.json",
    "artifact/reproduction-manifest.json",
    "artifact/source-inputs.manifest.json",
}

EXCLUDED_GENERATED_NAMES = {
    "backend-conformance.json",
    "build-manifest.json",
    "dynamic-dependency-manifest.json",
    "native-manifest.json",
    "native-replay-report.json",
}

EXCLUDED_PAPER_BUILD_SUFFIXES = {
    ".aux",
    ".bbl",
    ".blg",
    ".fdb_latexmk",
    ".fls",
    ".log",
    ".out",
    ".pdf",
}


def _logical(root: Path, path: Path) -> str:
    relative = path.relative_to(root)
    logical = PurePosixPath(*relative.parts).as_posix()
    if logical.startswith("/") or ".." in PurePosixPath(logical).parts:
        raise ValueError(f"invalid logical path: {logical}")
    return logical


def _excluded(logical: str) -> bool:
    if "__pycache__" in PurePosixPath(logical).parts or logical.endswith(".pyc"):
        return True
    if logical.startswith("paper/") and PurePosixPath(logical).suffix in EXCLUDED_PAPER_BUILD_SUFFIXES:
        return True
    if logical in EXCLUDED_EXACT or PurePosixPath(logical).name in EXCLUDED_BASENAMES:
        return True
    if PurePosixPath(logical).name in EXCLUDED_GENERATED_NAMES:
        return True
    return any(logical.startswith(prefix) for prefix in EXCLUDED_PREFIXES)


def _role(logical: str) -> str:
    path = PurePosixPath(logical)
    if logical == CONTRACT:
        return "semantic-contract"
    if path.name in {"Cargo.lock", "lean-toolchain", "lake-manifest.json", "go.sum"}:
        return "dependency-lock"
    if path.suffix == ".proto":
        return "protobuf-source"
    if logical.startswith("spec/schema/"):
        return "schema-definition"
    if logical.startswith("spec/run-config/"):
        return "run-configuration"
    if logical.startswith("spec/transformation-families/"):
        return "transformation-family"
    if logical.startswith("fixtures/"):
        return "fixture-specification"
    if logical.startswith("proof/"):
        return "proof-source"
    if logical.startswith("checker/"):
        return "checker-source"
    if logical.startswith("native/"):
        return "native-source"
    if logical.startswith("baselines/"):
        return "baseline-source"
    if logical.startswith("artifact/"):
        return "reproduction-source"
    if logical.startswith("docs/") or logical in {
        "README.md",
        "CONTRACT-AMENDMENT-2026-07-14.md",
    }:
        return "human-documentation"
    return "repository-control"


def build_manifest(root: Path) -> dict:
    contract = root / CONTRACT
    if not contract.is_file():
        raise ValueError(f"missing frozen contract: {CONTRACT}")
    actual_contract_hash = sha256_bytes(contract.read_bytes())
    if actual_contract_hash != APPROVED_CONTRACT_SHA256:
        raise ValueError(
            f"frozen contract hash mismatch: {actual_contract_hash} != "
            f"{APPROVED_CONTRACT_SHA256}"
        )

    entries = []
    paths: list[Path] = []
    for directory, names, files in os.walk(root):
        base = Path(directory)
        relative_directory = _logical(root, base) if base != root else ""
        names[:] = sorted(
            name
            for name in names
            if not _excluded(
                f"{relative_directory}/{name}/" if relative_directory else f"{name}/"
            )
        )
        paths.extend(base / name for name in sorted(files))
    for path in sorted(paths, key=str):
        logical = _logical(root, path)
        if _excluded(logical):
            continue
        mode = path.stat().st_mode
        entries.append(
            {
                "executable_bit": bool(mode & stat.S_IXUSR),
                "logical_path": logical,
                "role": _role(logical),
                "sha256": sha256_bytes(path.read_bytes()),
            }
        )

    entries.sort(key=lambda entry: entry["logical_path"])
    if len({entry["logical_path"] for entry in entries}) != len(entries):
        raise ValueError("duplicate logical source-input path")
    return {
        "entries": entries,
        "schema": "gluerift.source-inputs-manifest/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "source_tree_sha256": canonical_sha256(entries),
    }


def verify_entries(root: Path, manifest: dict) -> None:
    if manifest.get("schema") != "gluerift.source-inputs-manifest/v0.3.1a":
        raise ValueError("source-input manifest schema mismatch")
    entries = manifest.get("entries")
    if not isinstance(entries, list):
        raise ValueError("source-input manifest entries must be an array")
    if entries != sorted(entries, key=lambda entry: entry["logical_path"]):
        raise ValueError("source-input entries are not path-sorted")
    if canonical_sha256(entries) != manifest.get("source_tree_sha256"):
        raise ValueError("source_tree_sha256 mismatch")
    for entry in entries:
        logical = entry["logical_path"]
        path = root.joinpath(*PurePosixPath(logical).parts)
        if not path.is_file():
            raise ValueError(f"source input missing: {logical}")
        if sha256_bytes(path.read_bytes()) != entry["sha256"]:
            raise ValueError(f"source input hash mismatch: {logical}")
        executable = bool(path.stat().st_mode & stat.S_IXUSR)
        if executable != entry["executable_bit"]:
            raise ValueError(f"source input executable-bit mismatch: {logical}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--compare", type=Path)
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()

    if args.verify:
        verify_entries(root, load_json(args.verify))
    generated = build_manifest(root)
    if args.out:
        write_canonical(args.out, generated)
    if args.compare:
        checked = args.compare.read_bytes()
        expected = canonical_bytes(generated)
        if checked != expected:
            raise ValueError(f"source-input manifest is stale: {args.compare}")
    if not args.out and not args.compare and not args.verify:
        sys.stdout.buffer.write(canonical_bytes(generated))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError) as error:
        print(f"source-manifest error: {error}", file=sys.stderr)
        raise SystemExit(4)
