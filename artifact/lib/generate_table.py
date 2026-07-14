#!/usr/bin/env python3
"""Render the deterministic Core paper table from the unique result owner."""

from __future__ import annotations

import argparse
import csv
import io
import sys
from pathlib import Path
from typing import Any

from jcs import load_json


class TableError(ValueError):
    pass


PROPERTY_IDS = (
    "policy-soundness",
    "comparison-adequacy",
    "comparison-precision",
    "faithful-comparison",
    "target-non-amplification",
)


def _status(properties: dict[str, Any], property_id: str) -> str:
    value = properties.get(property_id)
    if value is None:
        value = properties.get(property_id.replace("-", "_"))
    if value is None and property_id == "target-non-amplification":
        value = properties.get("target_non_amplification_aggregate")
    if isinstance(value, str):
        return value
    if isinstance(value, dict):
        aggregate = value.get("aggregate_status", value.get("status"))
        if isinstance(aggregate, str):
            return aggregate
    raise TableError(f"missing typed property status: {property_id}")


def _six_roundtrips(value: Any) -> str:
    if not isinstance(value, dict) or len(value) != 6:
        raise TableError("six_roundtrip_statuses must contain exactly six laws")
    statuses = set(value.values())
    if statuses == {"proved-exhaustive"}:
        return "proved-exhaustive"
    if "tool-error" in statuses:
        return "tool-error"
    if "invalid" in statuses:
        return "invalid"
    if "unknown" in statuses:
        return "unknown"
    if "disproved" in statuses:
        return "disproved"
    return ",".join(f"{key}={value[key]}" for key in sorted(value))


TRANSFORM_FIELDS = (
    "classification", "base_alignment_status", "candidate_binding_status", "lawfulness_status",
    "transformation_sha256", "inverse_sha256", "action_domain_sha256",
    "transformed_context_sha256", "transformed_check_report_sha256", "harmful_witness_sha256",
)


def _transformation(row: dict[str, Any]) -> dict[str, str]:
    transformations = row.get("transformation_results", [])
    if not transformations:
        return {key: "not-applicable" for key in TRANSFORM_FIELDS}
    if not isinstance(transformations, list):
        raise TableError("transformation_results must be an array")
    output = {key: [] for key in TRANSFORM_FIELDS}
    for item in transformations:
        for key in TRANSFORM_FIELDS:
            output[key].append(str(item[key]))
    return {key: ";".join(values) for key, values in output.items()}


def _property_witness(row: dict[str, Any], property_id: str) -> str:
    values = sorted(
        item["witness_sha256"]
        for item in row.get("property_witnesses", [])
        if item["property_id"] == property_id
    )
    return ";".join(values) if values else "not-applicable"


def _baseline_status(value: Any, key: str) -> str:
    if value == "not-applicable" or value is None:
        return "not-applicable"
    if not isinstance(value, dict):
        raise TableError(f"{key} result has invalid shape")
    if key == "BL2":
        return _six_roundtrips(value["law_statuses"])
    parity = (
        value.get("property_parity_status"),
        value.get("witness_parity_status"),
    )
    if parity == ("proved-exhaustive", "proved-exhaustive"):
        return "proved-exhaustive"
    return "/".join(str(item) for item in parity)


def build_rows(results: dict[str, Any]) -> list[list[str]]:
    if results.get("schema") != "gluerift.results/v0.3.1a":
        raise TableError("result-owner schema mismatch")
    runs = results.get("runs")
    if not isinstance(runs, list) or not runs:
        raise TableError("result owner has no runs")
    if runs != sorted(runs, key=lambda row: row["run_id"]):
        raise TableError("result rows are not canonically ordered")

    output: list[list[str]] = []
    for row in runs:
        properties = row["property_statuses"]
        transform = _transformation(row)
        certification = row["certification"]
        match_coverage = row["match_coverage"]
        native = row.get("native_replay_result", "not-applicable")
        native_result = (
            native.get("ordinary_comparator_result", "not-applicable")
            if isinstance(native, dict)
            else "not-applicable"
        )
        output.append(
            [
                row["run_id"],
                row["validation_request_sha256"],
                row["check_report_sha256"],
                row["candidate_sha256"],
                row["comparator_kind"],
                row["profile"]["requested_profile"],
                row["profile"]["profile_property_consistency_status"],
                row["profile"]["safe_match_equality_status"],
                match_coverage["mode"],
                match_coverage["status"],
                str(match_coverage["source_comparison_domain_count"]),
                match_coverage["source_comparison_domain_sha256"],
                str(match_coverage["target_comparison_domain_count"]),
                match_coverage["target_comparison_domain_sha256"],
                str(match_coverage["matched_source_count"]),
                str(match_coverage["matched_target_count"]),
                row["policy_contract_status"],
                str(row["policy_vacuity_warning"]).lower(),
                str(certification["eligible"]).lower(),
                str(certification["granted"]).lower(),
                ";".join(certification["blocking_reasons"]) or "not-applicable",
                _six_roundtrips(row["six_roundtrip_statuses"]),
                *(_status(properties, item) for item in PROPERTY_IDS),
                *(_property_witness(row, item) for item in PROPERTY_IDS),
                row["bridge_statuses"]["carrier_source"],
                row["bridge_statuses"]["carrier_target"],
                row["bridge_statuses"]["selected_carrier_bridge"],
                *(transform[key] for key in TRANSFORM_FIELDS),
                _baseline_status(row.get("BL2_result", "not-applicable"), "BL2"),
                _baseline_status(row.get("BL4_result", "not-applicable"), "BL4"),
                native_result,
                native.get("replay_report_sha256", "not-applicable") if isinstance(native, dict) else "not-applicable",
                native.get("backend_conformance_sha256", "not-applicable") if isinstance(native, dict) else "not-applicable",
            ]
        )
    return output


HEADER = [
    "run_id",
    "validation_request_sha256",
    "check_report_sha256",
    "candidate_sha256",
    "comparator_kind",
    "profile",
    "profile_property_consistency_status",
    "safe_match_equality_status",
    "match_coverage_mode",
    "match_coverage_status",
    "source_comparison_domain_count",
    "source_comparison_domain_sha256",
    "target_comparison_domain_count",
    "target_comparison_domain_sha256",
    "matched_source_count",
    "matched_target_count",
    "policy_contract_status",
    "policy_vacuity_warning",
    "certificate_eligible",
    "certificate_granted",
    "certificate_blocking_reasons",
    "six_roundtrips",
    "policy_soundness",
    "comparison_adequacy",
    "comparison_precision",
    "faithful_comparison",
    "target_non_amplification",
    "policy_soundness_witness_sha256",
    "comparison_adequacy_witness_sha256",
    "comparison_precision_witness_sha256",
    "faithful_comparison_witness_sha256",
    "target_non_amplification_witness_sha256",
    "carrier_source_bridge_status",
    "carrier_target_bridge_status",
    "selected_carrier_bridge_status",
    "transformation_classification",
    "base_alignment_status",
    "candidate_binding_status",
    "transformation_lawfulness_status",
    "transformation_sha256",
    "inverse_sha256",
    "action_domain_sha256",
    "transformed_context_sha256",
    "transformed_check_report_sha256",
    "harmful_witness_sha256",
    "BL2_status",
    "BL4_parity_status",
    "native_ordinary_comparator",
    "native_replay_report_sha256",
    "native_backend_conformance_sha256",
]


def render_tsv(results: dict[str, Any]) -> bytes:
    stream = io.StringIO(newline="")
    writer = csv.writer(stream, dialect="excel-tab", lineterminator="\n")
    writer.writerow(HEADER)
    writer.writerows(build_rows(results))
    return stream.getvalue().encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--compare", type=Path)
    args = parser.parse_args()
    rendered = render_tsv(load_json(args.results))
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_bytes(rendered)
    if args.compare and args.compare.read_bytes() != rendered:
        raise TableError(f"paper table is stale: {args.compare}")
    if not args.out and not args.compare:
        sys.stdout.buffer.write(rendered)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, TableError, TypeError) as error:
        print(f"paper-table error: {error}", file=sys.stderr)
        raise SystemExit(4)
