#!/usr/bin/env python3
"""Reject claims whose guards exceed the canonical Core result owner."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path, PurePosixPath
from typing import Any

from jcs import canonical_sha256, load_json


class ClaimError(ValueError):
    pass


FORBIDDEN_LITERAL_WORDING = (
    "BL4 is weaker",
    "formally verified native",
    "proves program equivalence",
    "complete semantic",
    "arbitrary Go",
    "arbitrary Rust",
    "ecosystem prevalence",
)


def _property_status(row: dict[str, Any], property_id: str) -> str:
    value = row["property_statuses"].get(property_id)
    if value is None:
        value = row["property_statuses"].get(property_id.replace("-", "_"))
    if value is None and property_id == "target-non-amplification":
        value = row["property_statuses"].get("target_non_amplification_aggregate")
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        status = value.get("aggregate_status", value.get("status"))
        if isinstance(status, str):
            return status
    raise ClaimError(f"{row['run_id']}: missing property {property_id}")


def _resolve(root: Path, logical: str) -> Path:
    pure = PurePosixPath(logical)
    if pure.is_absolute() or not pure.parts or ".." in pure.parts:
        raise ClaimError(f"invalid evidence logical path: {logical}")
    path = root.joinpath(*pure.parts)
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise ClaimError(f"evidence path escapes repository: {logical}") from error
    return path


def _transformation(row: dict[str, Any], classification: str) -> dict[str, Any]:
    matches = [
        item
        for item in row.get("transformation_results", [])
        if item.get("classification") == classification
    ]
    if len(matches) != 1:
        raise ClaimError(
            f"{row['run_id']}: expected one {classification} transformation, got {len(matches)}"
        )
    return matches[0]


def validate(root: Path, claims: dict[str, Any], results: dict[str, Any]) -> None:
    if claims.get("schema") != "gluerift.claim-manifest/v0.3.1a":
        raise ClaimError("claim-manifest schema mismatch")
    if claims.get("semantic_contract_version") != "0.3.1a":
        raise ClaimError("claim-manifest semantic version mismatch")
    if results.get("schema") != "gluerift.results/v0.3.1a":
        raise ClaimError("result-owner schema mismatch")

    rows = {row["run_id"]: row for row in results.get("runs", [])}
    if len(rows) != len(results.get("runs", [])):
        raise ClaimError("duplicate result run ID")
    evidence = {entry["evidence_id"]: entry for entry in results.get("evidence_index", [])}
    evidence_by_hash = {entry["sha256"]: entry for entry in results.get("evidence_index", [])}
    if len(evidence) != len(results.get("evidence_index", [])):
        raise ClaimError("duplicate evidence ID in result owner")

    claim_rows = claims.get("claims")
    if not isinstance(claim_rows, list) or not claim_rows:
        raise ClaimError("claim manifest is empty")
    if claim_rows != sorted(claim_rows, key=lambda item: item["claim_id"]):
        raise ClaimError("claims are not canonically ordered")
    if len({item["claim_id"] for item in claim_rows}) != len(claim_rows):
        raise ClaimError("duplicate claim ID")

    for claim in claim_rows:
        claim_id = claim["claim_id"]
        wording = claim["permitted_wording"]
        if any(fragment.casefold() in wording.casefold() for fragment in FORBIDDEN_LITERAL_WORDING):
            raise ClaimError(f"{claim_id}: permitted wording contains a forbidden overstatement")
        forbidden = claim["forbidden_overstatement"]
        if not isinstance(forbidden, list) or not forbidden:
            raise ClaimError(f"{claim_id}: forbidden_overstatement must be a nonempty list")

        result_ref = claim["result"]
        if not isinstance(result_ref, dict) or result_ref.get("status") != "supported":
            raise ClaimError(f"{claim_id}: result must be explicitly supported")
        run_id = result_ref.get("run_id")
        if run_id not in rows:
            raise ClaimError(f"{claim_id}: unknown result run {run_id}")
        row = rows[run_id]

        required_ids = claim["required_evidence_ids"]
        if required_ids != sorted(set(required_ids)):
            raise ClaimError(f"{claim_id}: evidence IDs must be sorted and duplicate-free")
        for evidence_id in required_ids:
            item = evidence.get(evidence_id)
            if item is None:
                raise ClaimError(f"{claim_id}: unresolved evidence ID {evidence_id}")
            if item.get("profile") != "core":
                raise ClaimError(f"{claim_id}: non-Core evidence {evidence_id}")
            path = _resolve(root, item["logical_path"])
            if not path.is_file():
                raise ClaimError(f"{claim_id}: evidence file missing: {path}")
            document = load_json(path)
            if canonical_sha256(document) != item["sha256"]:
                raise ClaimError(f"{claim_id}: indexed evidence hash mismatch: {evidence_id}")
            if document.get("evidence_id") != evidence_id:
                raise ClaimError(f"{claim_id}: indexed evidence owner mismatch: {evidence_id}")
            if document.get("schema") != item.get("schema"):
                raise ClaimError(f"{claim_id}: indexed evidence schema mismatch: {evidence_id}")
            if document.get("status") in {"unknown", "tool-error", "invalid"}:
                raise ClaimError(f"{claim_id}: non-evidentiary status in {evidence_id}")

        if row["check_report_evidence_id"] not in required_ids:
            raise ClaimError(f"{claim_id}: canonical check report is not a required premise")
        check_entry = evidence[row["check_report_evidence_id"]]
        if check_entry["sha256"] != row["check_report_sha256"]:
            raise ClaimError(f"{claim_id}: result row does not bind its check evidence")

        guards = {
            "required_validation_request_sha256": row["validation_request_sha256"],
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
        for key, actual in guards.items():
            if claim[key] != actual:
                raise ClaimError(f"{claim_id}: guard {key} does not match result owner")
        if row["policy_contract_status"] == "policy-unconstrained":
            raise ClaimError(f"{claim_id}: policy-unconstrained evidence cannot support a claim")
        if row["profile"]["profile_property_consistency_status"] in {"unknown", "tool-error", "invalid"}:
            raise ClaimError(f"{claim_id}: non-evidentiary profile status")

        for property_id, expected in claim["required_property_statuses"].items():
            actual = _property_status(row, property_id)
            if actual != expected or actual in {"unknown", "tool-error", "invalid"}:
                raise ClaimError(f"{claim_id}: property guard mismatch for {property_id}")

        required_policy_witnesses = claim["required_policy_witness_ids"]
        available_policy_witnesses = set()
        for item in row.get("policy_witnesses", []):
            witness_hash = item.get("witness_sha256") if isinstance(item, dict) else item
            indexed = evidence_by_hash.get(witness_hash)
            if indexed is not None:
                available_policy_witnesses.add(indexed["evidence_id"])
        if not set(required_policy_witnesses).issubset(available_policy_witnesses):
            raise ClaimError(f"{claim_id}: required policy witness is absent")

        classification = claim["required_transformation_classification"]
        if classification == "not-applicable":
            if claim["required_base_alignment_status"] != "not-applicable" or claim["required_candidate_binding_status"] != "not-applicable":
                raise ClaimError(f"{claim_id}: non-transformation claim has binding guards")
        else:
            if classification != "lawful-harmful":
                raise ClaimError(f"{claim_id}: only lawful-harmful can support a laundering claim")
            transformed = _transformation(row, classification)
            for key in (
                "transformation_sha256",
                "inverse_sha256",
                "action_domain_sha256",
                "transformed_context_sha256",
                "transformed_check_report_sha256",
                "harmful_witness_sha256",
            ):
                if not isinstance(transformed.get(key), str) or len(transformed[key]) != 64:
                    raise ClaimError(f"{claim_id}: incomplete generated transformation binding: {key}")
            if transformed["base_alignment_status"] != claim["required_base_alignment_status"]:
                raise ClaimError(f"{claim_id}: base-alignment guard mismatch")
            if transformed["candidate_binding_status"] != claim["required_candidate_binding_status"]:
                raise ClaimError(f"{claim_id}: candidate-binding guard mismatch")
            if transformed["lawfulness_status"] != "proved-exhaustive":
                raise ClaimError(f"{claim_id}: transformation is not request-lawful")
            transformation_entries = [
                evidence[item]
                for item in required_ids
                if evidence[item]["schema"] == "gluerift.transformation-report/v0.3.1a"
            ]
            if len(transformation_entries) != 1:
                raise ClaimError(f"{claim_id}: transformation report premise is not unique")
            transformation_document = load_json(
                _resolve(root, transformation_entries[0]["logical_path"])
            )
            for key in (
                "classification", "base_alignment_status", "candidate_binding_status",
                "lawfulness_status", "transformation_sha256", "inverse_sha256",
                "action_domain_sha256", "transformed_context_sha256",
                "transformed_check_report_sha256", "harmful_witness_sha256",
            ):
                expected = transformed.get(key)
                if expected is not None and transformation_document.get(key) != expected:
                    raise ClaimError(f"{claim_id}: transformation premise mismatch: {key}")

        native_fixture_id = result_ref.get("native_fixture_id", "not-applicable")
        if native_fixture_id != "not-applicable":
            native = row.get("native_replay_result")
            if not isinstance(native, dict):
                raise ClaimError(f"{claim_id}: native replay binding is absent")
            if native.get("fixture_id") != native_fixture_id:
                raise ClaimError(f"{claim_id}: native fixture binding mismatch")
            if native.get("reference_run_id") != run_id:
                raise ClaimError(f"{claim_id}: native reference run mismatch")
            if native.get("reference_candidate_binding_status") != "proved-exhaustive":
                raise ClaimError(f"{claim_id}: native candidate binding is not proved")
            if native.get("ordinary_comparator_result") != "EQUAL":
                raise ClaimError(f"{claim_id}: native ordinary comparator is not equal")
            if claim.get("required_native_reference_bundle_sha256") != native.get(
                "reference_bundle_sha256"
            ):
                raise ClaimError(f"{claim_id}: native reference bundle hash guard mismatch")
            if native.get("reference_bundle_evidence_id") not in required_ids:
                raise ClaimError(f"{claim_id}: checker bundle is not a required premise")
            replay_entry = evidence.get(native.get("replay_report_evidence_id"))
            backend_entry = evidence.get(native.get("backend_conformance_evidence_id"))
            if replay_entry is None or backend_entry is None:
                raise ClaimError(f"{claim_id}: native replay premises are unresolved")
            replay = load_json(_resolve(root, replay_entry["logical_path"]))
            backend = load_json(_resolve(root, backend_entry["logical_path"]))
            if replay.get("ordinary_comparator_result") != "EQUAL":
                raise ClaimError(f"{claim_id}: native evidence does not record equality")
            if replay.get("property_statuses", {}).get("policy_soundness") != "disproved":
                raise ClaimError(f"{claim_id}: native evidence lacks the declared disagreement")
            if set(replay.get("six_roundtrip_statuses", {}).values()) != {"proved-exhaustive"}:
                raise ClaimError(f"{claim_id}: native evidence lacks all six round trips")
            if any(value != 0 for key, value in backend.items() if key.endswith("mismatch_count")):
                raise ClaimError(f"{claim_id}: backend conformance has mismatches")
            if any(
                backend.get(key) != []
                for key in (
                    "adapter_value_mismatches",
                    "comparator_truth_table_mismatches",
                    "roundtrip_truth_table_mismatches",
                )
            ):
                raise ClaimError(f"{claim_id}: backend differs from checker reference bundle")
        elif claim.get("required_native_reference_bundle_sha256") != "not-applicable":
            raise ClaimError(f"{claim_id}: non-native claim has a native bundle guard")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--claims", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    args = parser.parse_args()
    validate(args.root.resolve(), load_json(args.claims), load_json(args.results))
    print("claim guards: proved-exhaustive")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ClaimError, TypeError) as error:
        print(f"claim error: {error}", file=sys.stderr)
        raise SystemExit(4)
