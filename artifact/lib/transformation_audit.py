#!/usr/bin/env python3
"""Independently audit generated Core transformations and four-map conjugation.

This implementation intentionally shares no evaluator code with the Rust
semantic kernel.  It covers exactly the closed Minimal-Core Adapter/Type IR.
"""

from __future__ import annotations

import argparse
import itertools
import sys
from pathlib import Path
from typing import Any

from jcs import canonical_sha256, load_json


class TransformationAuditError(ValueError):
    pass


def enumerate_type(ty: dict[str, Any]) -> list[dict[str, Any]]:
    kind = ty["kind"]
    if kind == "unit":
        return [{"kind": "unit"}]
    if kind == "bool":
        return [{"kind": "bool", "value": False}, {"kind": "bool", "value": True}]
    if kind == "bounded-int":
        return [{"kind": kind, "value": value} for value in range(ty["min"], ty["max"] + 1)]
    if kind == "bit-vec":
        return [{"kind": kind, "value": value} for value in range(1 << ty["width"])]
    if kind == "sum":
        return [
            {"kind": kind, "variant": variant["name"], "payload": payload}
            for variant in ty["variants"]
            for payload in enumerate_type(variant["payload"])
        ]
    if kind == "product":
        fields = ty["fields"]
        domains = [enumerate_type(field["type"]) for field in fields]
        return [
            {
                "kind": kind,
                "fields": {field["name"]: value for field, value in zip(fields, values)},
            }
            for values in itertools.product(*domains)
        ]
    if kind == "object-result":
        return [
            {"kind": kind, "branch": branch, "value": value}
            for branch, branch_type in (("ok", ty["ok"]), ("err", ty["err"]))
            for value in enumerate_type(branch_type)
        ]
    raise TransformationAuditError(f"unsupported Core type: {kind}")


def evaluate(adapter: dict[str, Any], value: dict[str, Any]) -> dict[str, Any]:
    kind = adapter["kind"]
    if kind == "identity":
        return value
    if kind == "compose":
        return evaluate(adapter["second"], evaluate(adapter["first"], value))
    if kind == "enum-permutation":
        return {
            "kind": "sum",
            "payload": {"kind": "unit"},
            "variant": adapter["mapping"][value["variant"]],
        }
    if kind == "field-permutation":
        return {
            "kind": "product",
            "fields": {
                target: value["fields"][source]
                for target, source in adapter["mapping"].items()
            },
        }
    if kind == "sum-map":
        rule = adapter["variants"][value["variant"]]
        return {
            "kind": "sum",
            "payload": evaluate(rule["adapter"], value["payload"]),
            "variant": rule["target"],
        }
    if kind == "product-map":
        return {
            "kind": "product",
            "fields": {
                target: evaluate(rule["adapter"], value["fields"][rule["source"]])
                for target, rule in adapter["fields"].items()
            },
        }
    if kind == "result-map":
        branch = value["branch"]
        mapped_branch = branch if adapter["branch_mapping"] == "preserve" else (
            "err" if branch == "ok" else "ok"
        )
        return {
            "kind": "object-result",
            "branch": mapped_branch,
            "value": evaluate(adapter[branch], value["value"]),
        }
    if kind == "bounded-complement":
        return {
            "kind": "bounded-int",
            "value": adapter["min"] + adapter["max"] - value["value"],
        }
    if kind == "modular-affine":
        return {
            "kind": "bit-vec",
            "value": (
                adapter["scale"] * value["value"] + adapter["offset"]
            ) % (1 << adapter["width"]),
        }
    raise TransformationAuditError(f"unsupported Core adapter: {kind}")


def normalize(adapter: dict[str, Any]) -> dict[str, Any]:
    kind = adapter["kind"]
    if kind == "compose":
        stages: list[dict[str, Any]] = []

        def collect(item: dict[str, Any]) -> None:
            item = normalize(item)
            if item["kind"] == "compose":
                collect(item["first"])
                collect(item["second"])
            elif item["kind"] != "identity":
                stages.append(item)

        collect(adapter["first"])
        collect(adapter["second"])
        if not stages:
            return {"kind": "identity"}
        result = stages[0]
        for stage in stages[1:]:
            result = {"kind": "compose", "first": result, "second": stage}
        return result
    if kind == "sum-map":
        return {
            "kind": kind,
            "variants": {
                key: {"adapter": normalize(rule["adapter"]), "target": rule["target"]}
                for key, rule in adapter["variants"].items()
            },
        }
    if kind == "product-map":
        return {
            "kind": kind,
            "fields": {
                key: {"adapter": normalize(rule["adapter"]), "source": rule["source"]}
                for key, rule in adapter["fields"].items()
            },
        }
    if kind == "result-map":
        return {
            "kind": kind,
            "branch_mapping": adapter["branch_mapping"],
            "ok": normalize(adapter["ok"]),
            "err": normalize(adapter["err"]),
        }
    if kind == "modular-affine":
        modulus = 1 << adapter["width"]
        return {
            "kind": kind,
            "width": adapter["width"],
            "scale": adapter["scale"] % modulus,
            "offset": adapter["offset"] % modulus,
        }
    return adapter


def transformed_context(base: dict[str, Any], transformation: dict[str, Any], inverse: dict[str, Any]) -> dict[str, Any]:
    return {
        "carrier_type": base["carrier_type"],
        "schema": "gluerift.adapter-context/v0.3.1a",
        "source_decode": base["source_decode"],
        "source_encode": base["source_encode"],
        "source_type": base["source_type"],
        "target_decode": normalize(
            {"kind": "compose", "first": inverse, "second": base["target_decode"]}
        ),
        "target_encode": normalize(
            {"kind": "compose", "first": base["target_encode"], "second": transformation}
        ),
        "target_type": base["target_type"],
    }


def audit(root: Path, semantic_root: Path, registry_path: Path) -> int:
    registry = load_json(registry_path)
    rows = [item for item in registry["runs"] if item["transformation_report_required"]]
    if len(rows) != 8:
        raise TransformationAuditError("Core registry must contain exactly eight transformation runs")
    for row in rows:
        run_id = row["run_id"]
        run_dir = semantic_root / "runs" / run_id
        base = load_json(root / row["transformation_base_context_logical_path"])
        candidate = load_json(root / Path(row["request_logical_path"]).parent / "transformation.json")
        report = load_json(run_dir / "transformation.json")
        constructed = load_json(run_dir / "constructed-context.json")
        check = load_json(run_dir / "check.json")
        transformation = normalize(candidate["transformation_ir"])
        inverse = normalize(candidate["inverse_ir"])
        domain = enumerate_type(base["carrier_type"])

        if report["transformation_ir"] != transformation or report["inverse_ir"] != inverse:
            raise TransformationAuditError(f"{run_id}: normalized transformation provenance mismatch")
        for field, value in (
            ("transformation_sha256", transformation),
            ("inverse_sha256", inverse),
            ("action_domain_sha256", domain),
        ):
            if report[field] != canonical_sha256(value):
                raise TransformationAuditError(f"{run_id}: {field} mismatch")
        if report["action_domain"] != domain:
            raise TransformationAuditError(f"{run_id}: action domain is not All(C)")
        for value in domain:
            if evaluate(inverse, evaluate(transformation, value)) != value:
                raise TransformationAuditError(f"{run_id}: left inverse fails at {value}")
            if evaluate(transformation, evaluate(inverse, value)) != value:
                raise TransformationAuditError(f"{run_id}: right inverse fails at {value}")

        expected = transformed_context(base, transformation, inverse)
        if constructed != expected:
            raise TransformationAuditError(f"{run_id}: four-map conjugation AST mismatch")
        bindings = {
            "base_source_encode_sha256": base["source_encode"],
            "base_source_decode_sha256": base["source_decode"],
            "base_target_encode_sha256": base["target_encode"],
            "base_target_decode_sha256": base["target_decode"],
            "transformed_source_encode_sha256": constructed["source_encode"],
            "transformed_source_decode_sha256": constructed["source_decode"],
            "transformed_target_encode_sha256": constructed["target_encode"],
            "transformed_target_decode_sha256": constructed["target_decode"],
        }
        for field, value in bindings.items():
            if report[field] != canonical_sha256(value):
                raise TransformationAuditError(f"{run_id}: four-map hash mismatch at {field}")
        if report["transformed_context_sha256"] != canonical_sha256(constructed):
            raise TransformationAuditError(f"{run_id}: transformed context hash mismatch")
        if report["transformed_check_report_sha256"] != canonical_sha256(check):
            raise TransformationAuditError(f"{run_id}: transformed check hash mismatch")
        if report["classification"] != row["expected_transformation_classification"]:
            raise TransformationAuditError(f"{run_id}: registry classification mismatch")
        if report["roundtrip_statuses"] != row["expected_law_statuses"]:
            raise TransformationAuditError(f"{run_id}: requested-law classification mismatch")

        all_laws = set(report["roundtrip_statuses"].values()) == {"proved-exhaustive"}
        structural = all(
            report[key] == "proved-exhaustive"
            for key in (
                "well_typed_status",
                "inverse_check_status",
                "four_map_construction_status",
                "comparator_definedness_status",
            )
        )
        lawful = all_laws and structural
        sound = report["selected_property_statuses"]["policy-soundness"] == "proved-exhaustive"
        expected_class = (
            "law-breaking-or-inapplicable"
            if not lawful
            else "lawful-safe" if sound else "lawful-harmful"
        )
        if report["classification"] != expected_class:
            raise TransformationAuditError(f"{run_id}: independent classification mismatch")

        if row["expected_base_alignment_status"] == "proved-exhaustive":
            base_check = load_json(semantic_root / "runs" / f"{run_id}.base" / "check.json")
            if report["base_check_report_sha256"] != canonical_sha256(base_check):
                raise TransformationAuditError(f"{run_id}: aligned base report binding mismatch")
            for status in base_check["properties"].values():
                if isinstance(status, dict) and "status" in status and status["status"] != "proved-exhaustive":
                    raise TransformationAuditError(f"{run_id}: base is not aligned")
    return len(rows)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--semantic-root", type=Path, required=True)
    parser.add_argument("--registry", type=Path, required=True)
    args = parser.parse_args()
    count = audit(args.root.resolve(), args.semantic_root.resolve(), args.registry.resolve())
    print(f"independent transformation audit: proved-exhaustive ({count} candidates)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, TransformationAuditError, TypeError) as error:
        print(f"transformation audit error: {error}", file=sys.stderr)
        raise SystemExit(4)
