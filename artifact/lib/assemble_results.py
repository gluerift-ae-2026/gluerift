#!/usr/bin/env python3
"""Assemble the unique Core result owner from independently generated reports."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Iterable

from jcs import canonical_sha256, load_json, write_canonical


class ResultsError(ValueError):
    pass


def _logical(root: Path, path: Path) -> str:
    try:
        relative = path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise ResultsError(f"evidence path is outside release root: {path}") from error
    return PurePosixPath(*relative.parts).as_posix()


def _scan_evidence(root: Path, paths: Iterable[Path]) -> tuple[list[dict], dict[str, dict], dict[str, Path]]:
    files: set[Path] = set()
    for path in paths:
        if path.is_dir():
            files.update(item for item in path.rglob("*.json") if item.is_file())
        elif path.is_file() and path.suffix == ".json":
            files.add(path)
        else:
            raise ResultsError(f"evidence input is absent or not JSON: {path}")
    index: list[dict] = []
    by_hash: dict[str, dict] = {}
    paths_by_hash: dict[str, Path] = {}
    evidence_ids: set[str] = set()
    for path in sorted(files, key=lambda item: item.as_posix()):
        document = load_json(path)
        digest = canonical_sha256(document)
        if digest in by_hash:
            # Role-indexed build metadata may intentionally point at the same
            # content-addressed dependency set (for example, the harness and
            # target Rust binaries).  Only envelope-owning evidence must have
            # one unique byte owner; support objects are resolved by hash.
            if document.get("evidence_id") is not None or by_hash[digest].get("evidence_id") is not None:
                raise ResultsError(f"duplicate canonical evidence bytes: {path} and {paths_by_hash[digest]}")
        else:
            by_hash[digest] = document
            paths_by_hash[digest] = path
        evidence_id = document.get("evidence_id")
        if evidence_id is None:
            continue
        if evidence_id in evidence_ids:
            raise ResultsError(f"duplicate evidence ID: {evidence_id}")
        evidence_ids.add(evidence_id)
        index.append(
            {
                "evidence_id": evidence_id,
                "logical_path": _logical(root, path),
                "profile": "core",
                "schema": document.get("schema", "unknown"),
                "sha256": digest,
            }
        )
    index.sort(key=lambda item: item["evidence_id"])
    known_ids = {item["evidence_id"] for item in index}
    for item in index:
        document = by_hash[item["sha256"]]
        dependencies = document.get("dependency_evidence_ids", [])
        if dependencies != sorted(set(dependencies)):
            raise ResultsError(f"{item['evidence_id']}: noncanonical dependencies")
        missing = sorted(set(dependencies) - known_ids)
        if missing:
            raise ResultsError(f"{item['evidence_id']}: unresolved dependencies {missing}")
    return index, by_hash, paths_by_hash


def _baseline_map(document: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if document.get("schema") != "gluerift.baseline-results/v0.3.1a":
        raise ResultsError("baseline-results schema mismatch")
    rows = document.get("runs")
    if not isinstance(rows, list):
        raise ResultsError("baseline-results runs must be an array")
    output = {row["run_id"]: row for row in rows}
    if len(output) != len(rows):
        raise ResultsError("duplicate baseline result run")
    return output


def _attach_baseline(row: dict[str, Any], baseline: dict[str, Any] | None) -> None:
    if baseline is None:
        row["BL2_result"] = "not-applicable"
        row["BL4_result"] = "not-applicable"
        return
    row["BL2_result"] = baseline.get("BL2_result", "not-applicable")
    row["BL4_result"] = baseline.get("BL4_result", "not-applicable")


def _native_reports(index: dict[str, Any], by_hash: dict[str, dict]) -> dict[str, dict]:
    if index.get("schema") != "gluerift.native-output-index/v0.3.1a":
        raise ResultsError("native output index schema mismatch")
    output: dict[str, dict] = {}
    for item in index.get("fixtures", []):
        replay_hash = item["replay_report_sha256"]
        backend_hash = item["backend_conformance_sha256"]
        bundle_hash = item["reference_bundle_sha256"]
        replay = by_hash.get(replay_hash)
        backend = by_hash.get(backend_hash)
        bundle = by_hash.get(bundle_hash)
        if replay is None or backend is None or bundle is None:
            raise ResultsError(f"native index has unresolved report for {item['fixture_id']}")
        if (
            canonical_sha256(replay) != replay_hash
            or canonical_sha256(backend) != backend_hash
            or canonical_sha256(bundle) != bundle_hash
        ):
            raise ResultsError("native report hash mismatch")
        if replay["fixture_id"] != item["fixture_id"] or bundle["fixture_id"] != item["fixture_id"]:
            raise ResultsError("native fixture ID mismatch")
        if bundle.get("schema") != "gluerift.native-reference-bundle/v0.3.1a":
            raise ResultsError("native index resolves a non-bundle semantic authority")
        output[replay["reference_run_id"]] = {
            "backend_conformance_evidence_id": replay["backend_conformance_evidence_id"],
            "backend_conformance_sha256": backend_hash,
            "fixture_id": replay["fixture_id"],
            "ordinary_comparator_result": replay["ordinary_comparator_result"],
            "policy_soundness_status": replay["property_statuses"]["policy_soundness"],
            "reference_candidate_binding_status": replay["reference_candidate_binding_status"],
            "reference_candidate_sha256": replay["reference_candidate_sha256"],
            "reference_check_report_sha256": replay["reference_check_report_sha256"],
            "reference_bundle_evidence_id": replay["reference_bundle_evidence_id"],
            "reference_bundle_logical_path": item["reference_bundle_logical_path"],
            "reference_bundle_sha256": bundle_hash,
            "reference_run_id": replay["reference_run_id"],
            "replay_report_evidence_id": replay["evidence_id"],
            "replay_report_sha256": replay_hash,
            "six_roundtrip_statuses": replay["six_roundtrip_statuses"],
            "source_program_output": replay["source_program_output"],
            "target_program_output": replay["target_program_output"],
            "transported_source_as_target_native": replay[
                "transported_source_as_target_native"
            ],
            "violation_witness_path": replay["violation_witness"][
                "nested_adapter_path"
            ],
        }
    if set(output) != {"A01", "A02"}:
        raise ResultsError("native output must bind exactly E01->A01 and E02->A02")
    return output


def _all_six_proved(statuses: dict[str, str]) -> bool:
    return len(statuses) == 6 and set(statuses.values()) == {"proved-exhaustive"}


def _verify_result_edges(
    root: Path,
    row: dict[str, Any],
    by_hash: dict[str, dict],
    paths_by_hash: dict[str, Path],
) -> None:
    run_id = row["run_id"]
    check_hash = row["check_report_sha256"]
    check = by_hash.get(check_hash)
    if check is None or check.get("schema") != "gluerift.check-report/v0.3.1a":
        raise ResultsError(f"{run_id}: unresolved check report")
    if check.get("evidence_id") != row["check_report_evidence_id"]:
        raise ResultsError(f"{run_id}: check evidence ID mismatch")
    if _logical(root, paths_by_hash[check_hash]) != row["check_report_logical_path"]:
        raise ResultsError(f"{run_id}: check logical path mismatch")
    for key in ("candidate_sha256", "validation_request_sha256"):
        if check.get(key) != row[key]:
            raise ResultsError(f"{run_id}: check/result binding mismatch for {key}")
    for witness in row.get("property_witnesses", []):
        document = by_hash.get(witness["witness_sha256"])
        if document is None or document.get("schema") != "gluerift.witness/v0.3.1a":
            raise ResultsError(f"{run_id}: unresolved property witness")
        if document.get("witness_kind") != witness["witness_kind"]:
            raise ResultsError(f"{run_id}: property witness kind mismatch")
    for witness_hash in row.get("policy_witnesses", []):
        if witness_hash not in by_hash:
            raise ResultsError(f"{run_id}: unresolved policy witness")
    for transformed in row.get("transformation_results", []):
        report_hash = transformed["transformation_report_sha256"]
        report = by_hash.get(report_hash)
        if report is None or report.get("schema") != "gluerift.transformation-report/v0.3.1a":
            raise ResultsError(f"{run_id}: unresolved transformation report")
        for key in (
            "classification", "base_alignment_status", "candidate_binding_status",
            "lawfulness_status", "transformation_sha256", "inverse_sha256",
            "action_domain_sha256", "transformed_context_sha256",
            "transformed_check_report_sha256", "harmful_witness_sha256",
        ):
            if report.get(key) != transformed.get(key):
                raise ResultsError(f"{run_id}: transformation aggregate mismatch: {key}")
        if transformed["transformed_context_sha256"] not in by_hash:
            raise ResultsError(f"{run_id}: transformed context bytes are not evidence-bound")
        if transformed["transformed_check_report_sha256"] != check_hash:
            raise ResultsError(f"{run_id}: transformed check is not the result owner")
    for baseline_id in ("BL2", "BL4"):
        baseline = row.get(f"{baseline_id}_result")
        if baseline in (None, "not-applicable"):
            continue
        report = by_hash.get(baseline["report_sha256"])
        if report is None or report.get("evidence_id") != baseline["evidence_id"]:
            raise ResultsError(f"{run_id}: unresolved {baseline_id} report")
        if report.get("baseline_id") != baseline_id:
            raise ResultsError(f"{run_id}: baseline identity mismatch")
    if row.get("derivation_report_sha256") not in (None, "not-applicable"):
        derivation = by_hash.get(row["derivation_report_sha256"])
        if derivation is None or derivation.get("schema") != "gluerift.derivation-report/v0.3.1a":
            raise ResultsError(f"{run_id}: unresolved derivation report")


def _verify_native_binding(
    row: dict[str, Any],
    native: dict[str, Any],
    replay: dict[str, Any],
    backend: dict[str, Any],
    bundle: dict[str, Any],
) -> None:
    if native["reference_check_report_sha256"] != row["check_report_sha256"]:
        raise ResultsError(f"{row['run_id']}: native check-report binding mismatch")
    if native["reference_candidate_sha256"] != row["candidate_sha256"]:
        raise ResultsError(f"{row['run_id']}: native candidate binding mismatch")
    if native["reference_candidate_binding_status"] != "proved-exhaustive":
        raise ResultsError(f"{row['run_id']}: native candidate binding is not proved")
    transformations = row.get("transformation_results", [])
    if len(transformations) != 1:
        raise ResultsError(f"{row['run_id']}: native reference must have exactly one transformation")
    if replay["transformation_report_sha256"] != transformations[0]["transformation_report_sha256"]:
        raise ResultsError(f"{row['run_id']}: native transformation binding mismatch")
    bundle_bindings = {
        "reference_run_id": row["run_id"],
        "reference_check_report_sha256": row["check_report_sha256"],
        "candidate_context_sha256": row["candidate_sha256"],
        "transformation_report_sha256": transformations[0]["transformation_report_sha256"],
    }
    for key, expected in bundle_bindings.items():
        if bundle.get(key) != expected:
            raise ResultsError(f"{row['run_id']}: native reference bundle mismatch: {key}")
    if (
        replay.get("reference_bundle_sha256") != native["reference_bundle_sha256"]
        or backend.get("reference_bundle_sha256") != native["reference_bundle_sha256"]
        or replay.get("reference_bundle_evidence_id") != bundle.get("evidence_id")
        or backend.get("reference_bundle_evidence_id") != bundle.get("evidence_id")
    ):
        raise ResultsError(f"{row['run_id']}: native reports do not share the checker bundle")
    common = {
        "candidate_sha256": row["candidate_sha256"],
        "validation_request_sha256": row["validation_request_sha256"],
    }
    for key, expected in common.items():
        if replay.get(key, replay.get("context_sha256" if key == "candidate_sha256" else key)) != expected:
            raise ResultsError(f"{row['run_id']}: native common binding mismatch: {key}")
    if replay["ordinary_comparator_result"] != "EQUAL":
        raise ResultsError(f"{row['run_id']}: native ordinary comparator did not report EQUAL")
    if replay["comparator_kind"] != "target-native-exact":
        raise ResultsError(f"{row['run_id']}: wrong native comparator")
    if replay["property_statuses"].get("policy_soundness") != "disproved":
        raise ResultsError(f"{row['run_id']}: native policy soundness was not disproved")
    if not _all_six_proved(replay["six_roundtrip_statuses"]):
        raise ResultsError(f"{row['run_id']}: native six-roundtrip gate failed")
    for key, value in backend.items():
        if key.endswith("mismatch_count") and value != 0:
            raise ResultsError(f"{row['run_id']}: backend mismatch in {key}")
    for key in (
        "adapter_value_mismatches",
        "comparator_truth_table_mismatches",
        "roundtrip_truth_table_mismatches",
    ):
        if backend.get(key) != []:
            raise ResultsError(f"{row['run_id']}: backend mismatch entries in {key}")
    if row["run_id"] == "A02":
        if replay["violation_witness"].get("nested_adapter_path") != "output.policy.bounds.minimum":
            raise ResultsError("E02 nested field-role path is not actionable/canonical")


def assemble(args: argparse.Namespace) -> dict[str, Any]:
    root = args.root.resolve()
    fixtures = load_json(args.fixtures)
    if fixtures.get("schema") != "gluerift.fixture-results/v0.3.1a":
        raise ResultsError("fixture-results schema mismatch")
    fixture_rows = fixtures.get("runs")
    if not isinstance(fixture_rows, list) or not fixture_rows:
        raise ResultsError("fixture-results is empty")
    if fixture_rows != sorted(fixture_rows, key=lambda row: row["run_id"]):
        raise ResultsError("fixture results are not ordered by run ID")

    evidence_index, by_hash, paths_by_hash = _scan_evidence(root, args.evidence)
    baselines = _baseline_map(load_json(args.baselines))
    native_by_reference = _native_reports(load_json(args.native_index), by_hash)
    rows: list[dict[str, Any]] = []
    for original in fixture_rows:
        row = dict(original)
        _attach_baseline(row, baselines.get(row["run_id"]))
        _verify_result_edges(root, row, by_hash, paths_by_hash)
        native = native_by_reference.get(row["run_id"])
        if native is None:
            row["native_replay_result"] = "not-applicable"
        else:
            replay = by_hash[native["replay_report_sha256"]]
            backend = by_hash[native["backend_conformance_sha256"]]
            bundle = by_hash[native["reference_bundle_sha256"]]
            _verify_native_binding(row, native, replay, backend, bundle)
            row["native_replay_result"] = native
        rows.append(row)
    unconsumed_baselines = sorted(set(baselines) - {row["run_id"] for row in rows})
    if unconsumed_baselines:
        raise ResultsError(f"baseline rows have no semantic owner: {unconsumed_baselines}")

    proof_audit = load_json(args.proof_audit)
    if proof_audit.get("status") != "proved-exhaustive":
        raise ResultsError("Lean proof audit is not proved-exhaustive")
    return {
        "evidence_index": evidence_index,
        "native_replays": [native_by_reference[key] for key in sorted(native_by_reference)],
        "proof_audit": proof_audit,
        "runs": rows,
        "schema": "gluerift.results/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--baselines", type=Path, required=True)
    parser.add_argument("--native-index", type=Path, required=True)
    parser.add_argument("--proof-audit", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, action="append", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    write_canonical(args.out, assemble(args))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ResultsError, TypeError) as error:
        print(f"results error: {error}", file=sys.stderr)
        raise SystemExit(4)
