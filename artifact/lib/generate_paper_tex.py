#!/usr/bin/env python3
"""Generate the manuscript's categorical result tables from results.json."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

from jcs import canonical_sha256, load_json


class PaperTexError(ValueError):
    pass


OUTPUT_NAMES = (
    "core-results-table.tex",
    "regressions-table.tex",
    "native-table.tex",
    "bl4-delta-table.tex",
)


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise PaperTexError(message)


def _run_index(results: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if results.get("schema") != "gluerift.results/v0.3.1a":
        raise PaperTexError("result-owner schema mismatch")
    rows = results.get("runs")
    if not isinstance(rows, list) or not rows:
        raise PaperTexError("result owner has no runs")
    output = {row["run_id"]: row for row in rows}
    if len(output) != len(rows):
        raise PaperTexError("duplicate result-owner run ID")
    return output


def _property(row: dict[str, Any], property_id: str) -> str:
    properties = row["property_statuses"]
    value = properties.get(property_id, properties.get(property_id.replace("-", "_")))
    if not isinstance(value, str):
        raise PaperTexError(f"{row['run_id']}: missing property status {property_id}")
    return value


def _mark(status: str) -> str:
    if status == "proved-exhaustive":
        return r"\yes"
    if status == "disproved":
        return r"\no"
    raise PaperTexError(f"paper P/D cell has noncategorical status: {status}")


def _all_six_proved(row: dict[str, Any]) -> bool:
    statuses = row["six_roundtrip_statuses"]
    return isinstance(statuses, dict) and len(statuses) == 6 and set(statuses.values()) == {
        "proved-exhaustive"
    }


def _one_transformation(row: dict[str, Any]) -> dict[str, Any]:
    transformations = row.get("transformation_results")
    if not isinstance(transformations, list) or len(transformations) != 1:
        raise PaperTexError(f"{row['run_id']}: expected exactly one transformation")
    return transformations[0]


def _file(results_sha256: str, body: str) -> bytes:
    return (
        "% Generated from artifact/results/results.json; do not edit.\n"
        f"% Canonical result-owner SHA-256: {results_sha256}\n"
        f"{body}\n"
    ).encode("utf-8")


def _render_core(runs: dict[str, dict[str, Any]], digest: str) -> bytes:
    specifications = (
        ("A01.base", "A01.base", "enum aligned"),
        ("A01", "A01", "enum swap"),
        ("A02.base", "A02.base", "record aligned"),
        ("A02", "A02", "field swap"),
        ("A03.base", "A03.base", "scalar aligned"),
        ("A03", "A03", "complement"),
        ("A05.base", "A05.base", "result aligned"),
        ("A05", "A05", "result swap"),
        ("H01", "H01", "constructor rename"),
        ("H02", "H02", "correct reorder"),
        ("H04.tna", "H04.tna", "conservative target"),
        ("H04.exact", "H04.exact", "same candidate"),
    )
    rows: list[str] = []
    for index, (run_id, label, shape) in enumerate(specifications):
        row = runs.get(run_id)
        _require(row is not None, f"core paper row is missing: {run_id}")
        _require(_all_six_proved(row), f"{run_id}: paper RT cell is not proved")
        properties = [
            _mark(_property(row, property_id))
            for property_id in (
                "policy-soundness",
                "comparison-adequacy",
                "comparison-precision",
                "faithful-comparison",
            )
        ]
        transformations = row.get("transformation_results", [])
        if transformations:
            transformation = _one_transformation(row)
            _require(
                transformation["classification"] == "lawful-harmful",
                f"{run_id}: attack table row is not lawful-harmful",
            )
            class_or_certificate = "harmful"
        elif row["certification"]["granted"]:
            class_or_certificate = "cert."
        else:
            _require(run_id == "H04.exact", f"{run_id}: unexpected uncertified core row")
            class_or_certificate = "none"
        if index == 8:
            rows.append(r"\midrule")
        rows.append(
            f"{label} & {shape} & \\yes&"
            + "&".join(properties)
            + f"& {class_or_certificate}\\\\"
        )
    body = "\n".join(
        [
            r"\begin{tabular}{llcccccc}",
            r"\toprule",
            r"Case & Shape & RT & S & A & P & F & Class/cert.\\",
            r"\midrule",
            *rows,
            r"\bottomrule",
            r"\end{tabular}",
        ]
    )
    return _file(digest, body)


def _classification(runs: dict[str, dict[str, Any]], run_id: str, expected: str) -> None:
    row = runs.get(run_id)
    _require(row is not None, f"regression paper row is missing: {run_id}")
    transformation = _one_transformation(row)
    _require(
        transformation["classification"] == expected,
        f"{run_id}: classification differs from paper table",
    )


def _render_regressions(runs: dict[str, dict[str, Any]], digest: str) -> bytes:
    carrier = runs["V01.carrier"]
    target = runs["V01.target"]
    _require(_all_six_proved(carrier) and _all_six_proved(target), "V01 RT status mismatch")
    _require(_property(carrier, "policy-soundness") == "proved-exhaustive", "V01 carrier mismatch")
    _require(_property(target, "policy-soundness") == "disproved", "V01 target mismatch")
    _require(target["bridge_statuses"]["carrier_target"] == "disproved", "V01 bridge mismatch")

    v02 = runs["V02"]
    _require(_property(v02, "policy-soundness") == "invalid", "V02 status mismatch")
    _require(v02["match_coverage"]["status"] == "disproved", "V02 coverage mismatch")
    _require(_property(runs["V06"], "policy-soundness") == "unknown", "V06 status mismatch")

    v10 = runs["V10"]
    _require(
        [_property(v10, item) for item in (
            "policy-soundness", "comparison-adequacy", "comparison-precision", "faithful-comparison"
        )]
        == ["proved-exhaustive", "proved-exhaustive", "disproved", "disproved"],
        "V10 property vector mismatch",
    )
    vacuity = runs["policy-vacuity-conformance"]
    _require(
        vacuity["policy_contract_status"] == "policy-unconstrained"
        and vacuity["policy_vacuity_warning"] is True
        and vacuity["certification"]["granted"] is False,
        "policy-vacuity row mismatch",
    )
    _classification(runs, "T01.sigma1", "lawful-safe")
    _classification(runs, "T01.sigma2", "lawful-safe")
    _classification(runs, "T01.sigma1-compose-sigma2", "lawful-harmful")
    _classification(runs, "T02.sigma", "law-breaking-or-inapplicable")
    _require(
        _property(runs["T02.sigma"], "policy-soundness") == "proved-exhaustive"
        and not _all_six_proved(runs["T02.sigma"]),
        "T02 soundness/law distinction mismatch",
    )
    _require(
        runs["C01.exact"].get("derivation_status") == "proved-exhaustive"
        and runs["C01.tna"].get("derivation_status") == "proved-exhaustive",
        "C01 derivation status mismatch",
    )

    body = "\n".join(
        [
            r"\begin{tabular}{lll}",
            r"\toprule",
            r"Case & Outcome & Defect isolated\\",
            r"\midrule",
            r"V01.carrier & RT \yes, sound \yes & $E^C=\varnothing$\\",
            r"V01.target  & RT \yes, sound \no & $E^T$ unsafe; bridge \no\\",
            r"V02 & invalid & empty Match coverage\\",
            r"V06 & unknown & unsupported observer\\",
            r"V10 & S/A \yes; P/F \no & extra safe equality\\",
            r"Policy vacuity & no certificate & $\safe=U$ warning\\",
            r"T01.$\sigma_1$ & lawful-safe & asymmetric member\\",
            r"T01.$\sigma_2$ & lawful-safe & asymmetric member\\",
            r"T01.composite & lawful-harmful & policy-only non-closure\\",
            r"T02.$\sigma$ & law-breaking & sound but carrier RT \no\\",
            r"C01 & derivation \yes & total exact/preorder only\\",
            r"\bottomrule",
            r"\end{tabular}",
        ]
    )
    return _file(digest, body)


def _nested_bounds(value: dict[str, Any]) -> tuple[int, int]:
    try:
        fields = value["fields"]["output"]["fields"]["policy"]["fields"]["bounds"]["fields"]
        minimum = fields["minimum"]
        maximum = fields["maximum"]
    except (KeyError, TypeError) as error:
        raise PaperTexError("E02 native value does not have the paper's nested bounds shape") from error
    _require(minimum.get("kind") == maximum.get("kind") == "bounded-int", "E02 bounds type mismatch")
    return minimum["value"], maximum["value"]


def _native_display(fixture_id: str, value: dict[str, Any]) -> str:
    if fixture_id == "E01":
        _require(value.get("kind") == "sum", "E01 native value is not a sum")
        variant = value.get("variant")
        _require(isinstance(variant, str), "E01 native variant is absent")
        return variant
    if fixture_id == "E02":
        minimum, maximum = _nested_bounds(value)
        return f"min {minimum},max {maximum}"
    raise PaperTexError(f"unsupported native paper fixture: {fixture_id}")


def _render_native(results: dict[str, Any], digest: str) -> bytes:
    rows = results.get("native_replays")
    _require(isinstance(rows, list), "native replay summaries are absent")
    by_fixture = {row["fixture_id"]: row for row in rows}
    _require(set(by_fixture) == {"E01", "E02"}, "paper native table requires E01 and E02")
    rendered_rows: list[str] = []
    for fixture_id in ("E01", "E02"):
        row = by_fixture[fixture_id]
        _require(row["ordinary_comparator_result"] == "EQUAL", f"{fixture_id}: comparator mismatch")
        _require(row["policy_soundness_status"] == "disproved", f"{fixture_id}: soundness mismatch")
        _require(
            len(row["six_roundtrip_statuses"]) == 6
            and set(row["six_roundtrip_statuses"].values()) == {"proved-exhaustive"},
            f"{fixture_id}: six-roundtrip status mismatch",
        )
        source = _native_display(fixture_id, row["source_program_output"])
        target = _native_display(fixture_id, row["target_program_output"])
        transported = _native_display(fixture_id, row["transported_source_as_target_native"])
        rendered_rows.append(
            f"{fixture_id} & {source} & {target} & {transported} & "
            f"{row['ordinary_comparator_result']}\\\\"
        )
    body = "\n".join(
        [
            r"\begin{tabular}{lllll}",
            r"\toprule",
            r"Case & Source & Target & Transported & Result\\",
            r"\midrule",
            *rendered_rows,
            r"\bottomrule",
            r"\end{tabular}",
        ]
    )
    return _file(digest, body)


def _property_witness(row: dict[str, Any], property_id: str) -> str:
    matches = [
        item["witness_sha256"]
        for item in row.get("property_witnesses", [])
        if item["property_id"] == property_id
    ]
    _require(len(matches) == 1, f"{row['run_id']}: expected one {property_id} witness")
    return matches[0]


def _render_bl4_delta(runs: dict[str, dict[str, Any]], digest: str) -> bytes:
    row = runs["A02"]
    baseline = row.get("BL4_result")
    _require(isinstance(baseline, dict), "A02 BL4 result is absent")
    _require(
        _property(row, "policy-soundness") == "disproved"
        and baseline["property_statuses"]["policy-soundness"] == "disproved",
        "A02/BL4 policy verdict mismatch",
    )
    _require(baseline["witness_parity_status"] == "proved-exhaustive", "A02 witness parity mismatch")
    _require(
        _property_witness(row, "policy-soundness")
        == baseline["property_witnesses"]["policy-soundness"],
        "A02 first witness differs from BL4",
    )
    transformation = _one_transformation(row)
    _require(
        transformation["classification"] == "lawful-harmful"
        and transformation["inverse_sha256"] != "not-applicable"
        and transformation["action_domain_sha256"] != "not-applicable"
        and transformation["candidate_binding_status"] == "proved-exhaustive",
        "A02 generated transformation provenance is incomplete",
    )
    _require(
        row["bridge_statuses"]["carrier_target"] == "proved-exhaustive",
        "A02 transformed-context bridge is not proved",
    )
    native = row.get("native_replay_result")
    _require(isinstance(native, dict) and native["fixture_id"] == "E02", "A02 native binding is absent")
    _require(
        native["reference_candidate_sha256"] == row["candidate_sha256"],
        "A02 native candidate binding mismatch",
    )
    body = "\n".join(
        [
            r"\begin{tabular}{p{.48\columnwidth}p{.19\columnwidth}p{.22\columnwidth}}",
            r"\toprule",
            r"Output & BL4 & \sys\\",
            r"\midrule",
            r"Policy-soundness verdict & disproved & disproved\\",
            r"First endpoint witness & same & same\\",
            r"Nested semantic witness path & same & same\\",
            r"Generated harmful twist & no & enumerated\\",
            r"Normalized inverse/action domain & no & yes, $\all(C)$\\",
            r"Mechanical four-map conjugation & no & checked\\",
            r"Context-bound carrier bridge & no & checked\\",
            r"Native candidate binding & no & E02 bound\\",
            r"\bottomrule",
            r"\end{tabular}",
        ]
    )
    return _file(digest, body)


def render_all(results: dict[str, Any]) -> dict[str, bytes]:
    runs = _run_index(results)
    digest = canonical_sha256(results)
    return {
        "bl4-delta-table.tex": _render_bl4_delta(runs, digest),
        "core-results-table.tex": _render_core(runs, digest),
        "native-table.tex": _render_native(results, digest),
        "regressions-table.tex": _render_regressions(runs, digest),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument("--compare-dir", type=Path)
    args = parser.parse_args()
    if not args.out_dir and not args.compare_dir:
        parser.error("one of --out-dir or --compare-dir is required")
    rendered = render_all(load_json(args.results))
    if args.out_dir:
        args.out_dir.mkdir(parents=True, exist_ok=True)
        for name in OUTPUT_NAMES:
            (args.out_dir / name).write_bytes(rendered[name])
    if args.compare_dir:
        for name in OUTPUT_NAMES:
            path = args.compare_dir / name
            if not path.is_file() or path.read_bytes() != rendered[name]:
                raise PaperTexError(f"generated manuscript table is stale: {path}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, PaperTexError, TypeError) as error:
        print(f"paper-tex error: {error}", file=sys.stderr)
        raise SystemExit(4)
