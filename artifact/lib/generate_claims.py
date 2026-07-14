#!/usr/bin/env python3
"""Materialize guarded paper claims from a fixed claim specification."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

from jcs import load_json, write_canonical


class ClaimGenerationError(ValueError):
    pass


def _property_status(row: dict[str, Any], property_id: str) -> str:
    properties = row["property_statuses"]
    value = properties.get(property_id)
    if value is None:
        value = properties.get(property_id.replace("-", "_"))
    if value is None and property_id == "target-non-amplification":
        value = properties.get("target_non_amplification_aggregate")
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        status = value.get("aggregate_status", value.get("status"))
        if isinstance(status, str):
            return status
    raise ClaimGenerationError(f"{row['run_id']}: missing property {property_id}")


def _assert_guard(claim: dict[str, Any], key: str, actual: Any) -> None:
    if claim[key] != actual:
        raise ClaimGenerationError(
            f"{claim['claim_id']}: fixed claim guard {key}={claim[key]!r} does not match {actual!r}"
        )


def generate(spec: dict[str, Any], results: dict[str, Any]) -> dict[str, Any]:
    if spec.get("schema") != "gluerift.claim-spec/v0.3.1a":
        raise ClaimGenerationError("claim-spec schema mismatch")
    if results.get("schema") != "gluerift.results/v0.3.1a":
        raise ClaimGenerationError("result-owner schema mismatch")
    rows = {row["run_id"]: row for row in results["runs"]}
    evidence = {entry["evidence_id"]: entry for entry in results["evidence_index"]}
    evidence_by_hash = {entry["sha256"]: entry for entry in results["evidence_index"]}
    if len(evidence) != len(results["evidence_index"]) or len(evidence_by_hash) != len(results["evidence_index"]):
        raise ClaimGenerationError("evidence index is not one-to-one")

    claims = []
    for fixed in spec["claims"]:
        run_id = fixed["run_id"]
        if run_id not in rows:
            raise ClaimGenerationError(f"{fixed['claim_id']}: result row {run_id} is absent")
        row = rows[run_id]
        guard_values = {
            "required_profile": row["profile"]["requested_profile"],
            "required_profile_property_consistency_status": row["profile"]["profile_property_consistency_status"],
            "required_policy_status": row["policy_contract_status"],
            "required_comparator_kind": row["comparator_kind"],
            "required_match_coverage_mode": row["match_coverage"]["mode"],
            "required_match_coverage_status": row["match_coverage"]["status"],
            "required_safe_match_equality_status": row["profile"]["safe_match_equality_status"],
            "required_certification_eligible": row["certification"]["eligible"],
            "required_certification_granted": row["certification"]["granted"],
        }
        for key, actual in guard_values.items():
            _assert_guard(fixed, key, actual)
        for property_id, expected in fixed["required_property_statuses"].items():
            actual = _property_status(row, property_id)
            if actual != expected:
                raise ClaimGenerationError(
                    f"{fixed['claim_id']}: property {property_id} expected {expected}, got {actual}"
                )

        required_ids = {row["check_report_evidence_id"]}
        for witness in row.get("property_witnesses", []):
            item = evidence_by_hash.get(witness["witness_sha256"])
            if item is None:
                raise ClaimGenerationError(f"{fixed['claim_id']}: unresolved property witness")
            required_ids.add(item["evidence_id"])
        policy_witness_ids = []
        for witness_hash in row.get("policy_witnesses", []):
            item = evidence_by_hash.get(witness_hash)
            if item is None:
                raise ClaimGenerationError(f"{fixed['claim_id']}: unresolved policy witness")
            required_ids.add(item["evidence_id"])
            policy_witness_ids.append(item["evidence_id"])

        transformation_classification = fixed["required_transformation_classification"]
        if fixed["include_transformation"]:
            transformations = [
                item
                for item in row["transformation_results"]
                if item["classification"] == transformation_classification
            ]
            if len(transformations) != 1:
                raise ClaimGenerationError(f"{fixed['claim_id']}: transformation result is not unique")
            transformed = transformations[0]
            report = evidence_by_hash.get(transformed["transformation_report_sha256"])
            if report is None:
                raise ClaimGenerationError(f"{fixed['claim_id']}: transformation report is absent")
            required_ids.add(report["evidence_id"])
            _assert_guard(fixed, "required_base_alignment_status", transformed["base_alignment_status"])
            _assert_guard(fixed, "required_candidate_binding_status", transformed["candidate_binding_status"])
        elif row.get("transformation_results"):
            raise ClaimGenerationError(f"{fixed['claim_id']}: transformation evidence exists but spec excludes it")

        if fixed["include_bl4"]:
            bl4 = row.get("BL4_result")
            if not isinstance(bl4, dict):
                raise ClaimGenerationError(f"{fixed['claim_id']}: BL4 result is absent")
            if bl4["property_parity_status"] != "proved-exhaustive" or bl4["witness_parity_status"] != "proved-exhaustive":
                raise ClaimGenerationError(f"{fixed['claim_id']}: BL4 parity is not proved")
            required_ids.add(bl4["evidence_id"])

        native_fixture = fixed["include_native_fixture"]
        native_reference_bundle_sha256 = "not-applicable"
        if native_fixture != "not-applicable":
            native = row.get("native_replay_result")
            if not isinstance(native, dict) or native["fixture_id"] != native_fixture:
                raise ClaimGenerationError(f"{fixed['claim_id']}: native replay is absent")
            required_ids.add(native["replay_report_evidence_id"])
            required_ids.add(native["backend_conformance_evidence_id"])
            required_ids.add(native["reference_bundle_evidence_id"])
            native_reference_bundle_sha256 = native["reference_bundle_sha256"]
        if fixed["include_proof"]:
            if "lean-proof-audit" not in evidence:
                raise ClaimGenerationError(f"{fixed['claim_id']}: Lean proof audit evidence is absent")
            required_ids.add("lean-proof-audit")

        claims.append(
            {
                "claim_id": fixed["claim_id"],
                "forbidden_overstatement": fixed["forbidden_overstatement"],
                "permitted_wording": fixed["permitted_wording"],
                "required_base_alignment_status": fixed["required_base_alignment_status"],
                "required_candidate_binding_status": fixed["required_candidate_binding_status"],
                "required_certification_eligible": fixed["required_certification_eligible"],
                "required_certification_granted": fixed["required_certification_granted"],
                "required_comparator_kind": fixed["required_comparator_kind"],
                "required_evidence_ids": sorted(required_ids),
                "required_match_coverage_mode": fixed["required_match_coverage_mode"],
                "required_match_coverage_status": fixed["required_match_coverage_status"],
                "required_native_reference_bundle_sha256": native_reference_bundle_sha256,
                "required_policy_status": fixed["required_policy_status"],
                "required_policy_witness_ids": sorted(policy_witness_ids),
                "required_profile": fixed["required_profile"],
                "required_profile_property_consistency_status": fixed["required_profile_property_consistency_status"],
                "required_property_statuses": fixed["required_property_statuses"],
                "required_safe_match_equality_status": fixed["required_safe_match_equality_status"],
                "required_transformation_classification": transformation_classification,
                "required_validation_request_sha256": row["validation_request_sha256"],
                "result": {
                    "native_fixture_id": native_fixture,
                    "run_id": run_id,
                    "status": "supported",
                },
            }
        )
    claims.sort(key=lambda item: item["claim_id"])
    return {
        "claims": claims,
        "schema": "gluerift.claim-manifest/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--spec", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    write_canonical(args.out, generate(load_json(args.spec), load_json(args.results)))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ClaimGenerationError, TypeError) as error:
        print(f"claim-generation error: {error}", file=sys.stderr)
        raise SystemExit(4)
