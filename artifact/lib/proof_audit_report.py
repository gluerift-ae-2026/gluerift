#!/usr/bin/env python3
"""Bind the successful Lean L1--L9 build, axiom audit, and hygiene check."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

from jcs import canonical_sha256, load_json, sha256_bytes, write_canonical


class ProofAuditError(ValueError):
    pass


REQUIRED_MODULES = tuple(f"L{index}" for index in range(1, 10))


def build(args: argparse.Namespace) -> dict:
    root = args.proof_root.resolve()
    lean_files = sorted(
        path
        for path in root.rglob("*.lean")
        if not ({".toolchain", ".lake"} & set(path.relative_to(root).parts))
    )
    if not lean_files:
        raise ProofAuditError("proof tree has no Lean sources")
    source_entries = []
    names = set()
    for path in lean_files:
        text = path.read_text(encoding="utf-8")
        lowered = text.casefold()
        for forbidden in ("sorry", "admit"):
            if re.search(rf"\b{forbidden}\b", lowered):
                raise ProofAuditError(f"placeholder `{forbidden}` in {path}")
        logical = PurePosixPath(*path.relative_to(root).parts).as_posix()
        source_entries.append({"logical_path": f"proof/{logical}", "sha256": sha256_bytes(path.read_bytes())})
        names.add(path.stem)
    for prefix in REQUIRED_MODULES:
        if not any(name.startswith(prefix) for name in names):
            raise ProofAuditError(f"missing mechanized group {prefix}")

    audit = args.audit_log.read_text(encoding="utf-8")
    if "Lean proof hygiene and axiom audit passed." not in audit:
        raise ProofAuditError("Lean audit log lacks the proof-hygiene success marker")
    for prefix in REQUIRED_MODULES:
        if prefix not in audit:
            raise ProofAuditError(f"Lean audit log lacks {prefix}")
    if "depends on axioms" in audit and "does not depend on any axioms" not in audit:
        raise ProofAuditError("Lean axiom audit reported a dependency")

    lean = args.lean_executable.resolve()
    if not lean.is_file():
        raise ProofAuditError("Lean executable is absent")
    version = subprocess.run(
        [str(lean), "--version"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    ).stdout.strip()
    configuration = load_json(args.run_configuration)
    proof_hash = canonical_sha256(source_entries)
    return {
        "axiom_audit_log_sha256": sha256_bytes(args.audit_log.read_bytes()),
        "candidate_sha256": proof_hash,
        "comparator_spec_sha256": "not-applicable",
        "dependency_evidence_ids": [],
        "endpoint_policy_sha256": "not-applicable",
        "evidence_id": "lean-proof-audit",
        "hygiene_status": "proved-exhaustive",
        "lean_executable_sha256": sha256_bytes(lean.read_bytes()),
        "lean_version": version,
        "mechanized_groups": list(REQUIRED_MODULES),
        "run_configuration_sha256": canonical_sha256(configuration),
        "schema": "gluerift.proof-audit/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "source_entries": source_entries,
        "status": "proved-exhaustive",
        "tool_build_sha256": sha256_bytes(lean.read_bytes()),
        "types_sha256": "not-applicable",
        "validation_request_sha256": "not-applicable",
        "validation_scope_sha256": "not-applicable",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--proof-root", type=Path, required=True)
    parser.add_argument("--audit-log", type=Path, required=True)
    parser.add_argument("--lean-executable", type=Path, required=True)
    parser.add_argument("--run-configuration", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    write_canonical(args.out, build(args))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ProofAuditError, subprocess.CalledProcessError) as error:
        print(f"proof-audit error: {error}", file=sys.stderr)
        raise SystemExit(4)
