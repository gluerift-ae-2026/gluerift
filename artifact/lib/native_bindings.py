#!/usr/bin/env python3
"""Derive the exact E01/E02 native reference bindings from semantic evidence."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from jcs import canonical_sha256, load_json, write_canonical


class BindingError(ValueError):
    pass


PAIRS = (("E01", "A01"), ("E02", "A02"))


def build(source_manifest_path: Path, semantic_root: Path) -> dict:
    source_manifest = load_json(source_manifest_path)
    fixture_results = load_json(semantic_root / "fixture-results.json")
    rows = {row["run_id"]: row for row in fixture_results["runs"]}
    references = []
    for fixture_id, run_id in PAIRS:
        if run_id not in rows:
            raise BindingError(f"missing semantic reference row {run_id}")
        row = rows[run_id]
        run_dir = semantic_root / "runs" / run_id
        check_path = run_dir / "check.json"
        transformation_path = run_dir / "transformation.json"
        context_path = run_dir / "constructed-context.json"
        bundle_path = run_dir / "native-reference-bundle.json"
        check = load_json(check_path)
        transformation = load_json(transformation_path)
        context = load_json(context_path)
        bundle = load_json(bundle_path)
        check_hash = canonical_sha256(check)
        transformation_hash = canonical_sha256(transformation)
        context_hash = canonical_sha256(context)
        bundle_hash = canonical_sha256(bundle)
        if check_hash != row["check_report_sha256"]:
            raise BindingError(f"{run_id}: aggregate/check report hash mismatch")
        if context_hash != row["candidate_sha256"] or context_hash != transformation["transformed_context_sha256"]:
            raise BindingError(f"{run_id}: transformed candidate hash mismatch")
        if transformation_hash != row["transformation_results"][0]["transformation_report_sha256"]:
            raise BindingError(f"{run_id}: transformation report hash mismatch")
        if transformation["candidate_binding_status"] != "proved-exhaustive":
            raise BindingError(f"{run_id}: transformed candidate binding is not proved")
        if (
            bundle["fixture_id"] != fixture_id
            or bundle["reference_run_id"] != run_id
            or bundle["reference_check_report_sha256"] != check_hash
            or bundle["reference_check_evidence_id"] != check["evidence_id"]
            or bundle["candidate_context_sha256"] != context_hash
            or bundle["transformation_report_sha256"] != transformation_hash
            or bundle["source_inputs_manifest_sha256"] != canonical_sha256(source_manifest)
            or bundle["source_tree_sha256"] != source_manifest["source_tree_sha256"]
            or len(bundle["six_roundtrip_truth_tables"]) != 6
        ):
            raise BindingError(f"{run_id}: checker native-reference bundle binding mismatch")
        references.append(
            {
                "comparator_spec_sha256": check["comparator_spec_sha256"],
                "endpoint_policy_sha256": check["endpoint_policy_sha256"],
                "fixture_id": fixture_id,
                "reference_candidate_sha256": context_hash,
                "reference_check_evidence_id": check["evidence_id"],
                "reference_check_report_sha256": check_hash,
                "reference_bundle_logical_path": f"artifact/evidence/semantic/runs/{run_id}/native-reference-bundle.json",
                "reference_bundle_sha256": bundle_hash,
                "reference_run_id": run_id,
                "run_configuration_sha256": check["run_configuration_sha256"],
                "transformation_report_sha256": transformation_hash,
                "transformed_context_sha256": context_hash,
                "types_sha256": check["types_sha256"],
                "validation_request_sha256": check["validation_request_sha256"],
                "validation_scope_sha256": check["validation_scope_sha256"],
            }
        )
    references.sort(key=lambda item: item["fixture_id"])
    return {
        "references": references,
        "schema": "gluerift.native-reference-bindings/v0.3.1a",
        "source_inputs_manifest_sha256": canonical_sha256(source_manifest),
        "source_tree_sha256": source_manifest["source_tree_sha256"],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-manifest", type=Path, required=True)
    parser.add_argument("--semantic-root", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    write_canonical(args.out, build(args.source_manifest, args.semantic_root))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (IndexError, KeyError, OSError, BindingError) as error:
        print(f"native-binding error: {error}", file=sys.stderr)
        raise SystemExit(4)
