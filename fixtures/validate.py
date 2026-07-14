#!/usr/bin/env python3
"""Validate authored GlueRift Core fixture declarations without evaluating them."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "fixtures/registry.json"
RUN_CONFIG_PATH = ROOT / "spec/run-config/core-v0.3.1a.json"
FAMILY_PATH = ROOT / "spec/transformation-families/core-structural-v0.3.1a.json"

REGISTRY_FIELDS = {
    "run_id",
    "fixture_kind",
    "context_logical_path",
    "transformation_base_context_logical_path",
    "scope_logical_path",
    "policy_logical_path",
    "request_logical_path",
    "request_id",
    "validation_request_sha256",
    "profile",
    "required_law_ids",
    "required_properties",
    "required_properties_sha256",
    "required_bridge_ids",
    "required_transformation_family_sha256",
    "comparator_spec_sha256",
    "run_configuration_sha256",
    "expected_profile_property_consistency",
    "match_coverage_mode",
    "expected_match_coverage_status",
    "expected_safe_match_equality_status",
    "expected_certificate_eligibility",
    "expected_certificate_granted",
    "expected_comparator_definedness_status",
    "expected_law_statuses",
    "expected_property_statuses",
    "expected_bridge_statuses",
    "expected_policy_contract_status",
    "transformation_report_required",
    "transformation_sha256",
    "expected_transformation_classification",
    "expected_candidate_binding_status",
    "expected_base_alignment_status",
    "required_witness_kinds",
    "bl2_paired",
    "bl4_paired",
    "native_replay_id",
}

ALL_LAWS = [
    "source-native",
    "target-native",
    "source-carrier",
    "target-carrier",
    "source-full-transport",
    "target-full-transport",
]

LAW_FLAGS = [
    ("source-native", "source_native_roundtrip"),
    ("target-native", "target_native_roundtrip"),
    ("source-carrier", "source_carrier_roundtrip"),
    ("target-carrier", "target_carrier_roundtrip"),
    ("source-full-transport", "source_full_transport"),
    ("target-full-transport", "target_full_transport"),
]

RUN_IDS = {
    "A01", "A01.base", "A02", "A02.base", "A03", "A03.base", "A05", "A05.base",
    "H01", "H02", "H04.tna", "H04.exact",
    "V01.carrier", "V01.target", "V02", "V06", "V10", "policy-vacuity-conformance",
    "T01.sigma1", "T01.sigma2", "T01.sigma1-compose-sigma2", "T02.sigma",
    "C01.exact", "C01.tna",
}

TRANSFORMED = {
    "A01", "A02", "A03", "A05",
    "T01.sigma1", "T01.sigma2", "T01.sigma1-compose-sigma2", "T02.sigma",
}

BL2_RUNS = {"A01", "A02", "A03", "A05"}
BL4_RUNS = {
    "A01", "A02", "A03", "A05", "H01", "H02", "H04.tna", "H04.exact",
    "V01.carrier", "V01.target", "V02", "V06", "V10",
    "C01.exact", "C01.tna", "policy-vacuity-conformance",
}


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":"), sort_keys=True).encode("utf-8")


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def load(path: Path, *, require_canonical_bytes: bool = True) -> Any:
    raw = path.read_bytes()
    value = json.loads(raw)
    if require_canonical_bytes and raw != canonical_bytes(value):
        raise AssertionError(f"noncanonical JSON bytes: {path.relative_to(ROOT)}")
    return value


def assert_no_oracle_input(path: Path, value: Any) -> None:
    def visit(item: Any, location: str) -> None:
        if isinstance(item, dict):
            for key, child in item.items():
                if key.startswith("expected_"):
                    raise AssertionError(f"oracle key in evaluator input {path.relative_to(ROOT)} at {location}.{key}")
                visit(child, f"{location}.{key}")
        elif isinstance(item, list):
            for index, child in enumerate(item):
                visit(child, f"{location}[{index}]")
    visit(value, "$")


def validate() -> None:
    # These two source files are owned by spec/; their semantic JCS hashes are
    # validated here while their physical-byte audit is owned by artifact/.
    run_config = load(RUN_CONFIG_PATH, require_canonical_bytes=False)
    family = load(FAMILY_PATH, require_canonical_bytes=False)
    registry = load(REGISTRY_PATH)
    assert registry["schema"] == "gluerift.fixture-registry/v0.3.1a"
    assert registry["semantic_contract_version"] == "0.3.1a"
    run_config_hash = digest(run_config)
    family_hash = digest(family)

    rows = registry["runs"]
    assert [row["run_id"] for row in rows] == sorted(RUN_IDS)
    assert {row["run_id"] for row in rows} == RUN_IDS
    assert len(rows) == len(RUN_IDS)

    for row in rows:
        run_id = row["run_id"]
        assert set(row) == REGISTRY_FIELDS, f"{run_id}: registry field mismatch"
        assert row["run_configuration_sha256"] == run_config_hash
        assert row["required_transformation_family_sha256"] == family_hash
        assert row["bl2_paired"] == (run_id in BL2_RUNS)
        assert row["bl4_paired"] == (run_id in BL4_RUNS)

        scope_path = ROOT / row["scope_logical_path"]
        policy_path = ROOT / row["policy_logical_path"]
        request_path = ROOT / row["request_logical_path"]
        context_path = ROOT / row["context_logical_path"]
        scope = load(scope_path)
        policy = load(policy_path)
        request = load(request_path)
        for path, value in [(scope_path, scope), (policy_path, policy), (request_path, request)]:
            assert_no_oracle_input(path, value)

        assert request["request_id"] == row["request_id"]
        assert request["profile"] == row["profile"]
        assert digest(request) == row["validation_request_sha256"]
        assert digest(request["required_properties"]) == row["required_properties_sha256"]
        assert request["required_properties"] == row["required_properties"]
        assert request["required_bridges"] == row["required_bridge_ids"]
        assert request["required_transformation_family_sha256"] == family_hash
        assert request["run_configuration_sha256"] == run_config_hash
        assert request["validation_scope_sha256"] == digest(scope)
        assert request["endpoint_policy_sha256"] == digest(policy)
        assert digest(scope["comparator"]) == row["comparator_spec_sha256"]
        assert policy["match_coverage"] == row["match_coverage_mode"]
        asserted_laws = [name for name, field in LAW_FLAGS if request["required_laws"][field]]
        assert asserted_laws == row["required_law_ids"]

        if run_id in TRANSFORMED:
            assert row["transformation_report_required"] is True
            assert row["context_logical_path"].startswith("artifact/staging/generated-contexts/")
            assert not context_path.exists(), f"authored transformed context exists: {context_path}"
            base_path = ROOT / row["transformation_base_context_logical_path"]
            load(base_path)
            transformation_path = request_path.parent / "transformation.json"
            transformation = load(transformation_path)
            assert_no_oracle_input(transformation_path, transformation)
            assert digest(transformation["transformation_ir"]) == row["transformation_sha256"]
            assert row["expected_candidate_binding_status"] == "proved-exhaustive"
            if run_id.startswith("A"):
                assert row["expected_base_alignment_status"] == "proved-exhaustive"
            else:
                assert row["expected_base_alignment_status"] == "not-required"
        else:
            assert row["transformation_report_required"] is False
            load(context_path)
            assert row["transformation_base_context_logical_path"] == "not-applicable"
            assert row["transformation_sha256"] == "not-applicable"
            assert row["expected_transformation_classification"] == "not-applicable"
            assert row["expected_candidate_binding_status"] == "not-applicable"
            assert row["expected_base_alignment_status"] == "not-applicable"

        if run_id.startswith("A"):
            assert row["required_law_ids"] == ALL_LAWS
            assert len(row["required_properties"]) == 4
            if run_id.endswith(".base"):
                assert all(value == "proved-exhaustive" for value in row["expected_property_statuses"].values() if value != "not-requested")
                assert row["expected_certificate_granted"] is True
            else:
                assert all(row["expected_property_statuses"][key] == "disproved" for key in ["policy-soundness", "comparison-adequacy", "comparison-precision", "faithful-comparison"])
                assert row["expected_transformation_classification"] == "lawful-harmful"
                assert row["expected_certificate_granted"] is False

    by_id = {row["run_id"]: row for row in rows}
    assert by_id["V01.carrier"]["expected_property_statuses"]["policy-soundness"] == "proved-exhaustive"
    assert by_id["V01.target"]["expected_property_statuses"]["policy-soundness"] == "disproved"
    assert by_id["V02"]["expected_profile_property_consistency"] == "invalid"
    assert by_id["V06"]["expected_property_statuses"]["policy-soundness"] == "unknown"
    assert by_id["V10"]["expected_property_statuses"]["policy-soundness"] == "proved-exhaustive"
    assert by_id["V10"]["expected_property_statuses"]["comparison-precision"] == "disproved"
    assert by_id["policy-vacuity-conformance"]["expected_policy_contract_status"] == "policy-unconstrained"
    assert by_id["H04.tna"]["required_properties"][-1] == {"kind": "target-non-amplification", "dimension_ids": ["policy-level"]}
    assert by_id["H04.exact"]["expected_property_statuses"] == {
        "policy-soundness": "disproved",
        "comparison-adequacy": "proved-exhaustive",
        "comparison-precision": "disproved",
        "faithful-comparison": "disproved",
        "target-non-amplification": "not-requested",
    }
    assert by_id["T01.sigma1"]["expected_transformation_classification"] == "lawful-safe"
    assert by_id["T01.sigma2"]["expected_transformation_classification"] == "lawful-safe"
    assert by_id["T01.sigma1-compose-sigma2"]["expected_transformation_classification"] == "lawful-harmful"
    assert by_id["T02.sigma"]["expected_transformation_classification"] == "law-breaking-or-inapplicable"
    assert by_id["T02.sigma"]["expected_law_statuses"]["target-carrier"] == "disproved"

    a02_transform = load(ROOT / "fixtures/attacks/A02/transformation.json")
    assert a02_transform["generation_parent_path"] == ["output", "policy", "bounds"]
    a02_policy = load(ROOT / "fixtures/attacks/A02/policy.json")
    assert a02_policy["safe_dimensions"] == ["00-minimum-role", "01-maximum-role"]
    assert a02_policy["dimensions"][0]["source_observer"]["path"] == ["output", "policy", "bounds", "minimum"]
    assert a02_policy["dimensions"][1]["source_observer"]["path"] == ["output", "policy", "bounds", "maximum"]
    assert by_id["A01"]["native_replay_id"] == "E01"
    assert by_id["A02"]["native_replay_id"] == "E02"
    assert by_id["A03"]["native_replay_id"] == "not-applicable"
    assert by_id["A05"]["native_replay_id"] == "not-applicable"

    bl2 = load(ROOT / "baselines/BL2/config.json")
    bl4 = load(ROOT / "baselines/BL4/config.json")
    assert set(bl2["paired_run_ids"]) == BL2_RUNS
    assert set(bl4["paired_run_ids"]) == BL4_RUNS
    source_index = load(ROOT / "fixtures/source-index.json")
    assert source_index["fixture_registry_sha256"] == digest(registry)
    assert source_index["run_ids"] == sorted(RUN_IDS)

    json_files = sorted((ROOT / "fixtures").rglob("*.json")) + sorted((ROOT / "baselines").rglob("*.json"))
    for path in json_files:
        load(path)
    sys.path.insert(0, str(ROOT / "artifact/lib"))
    from schema_check import SchemaCatalog
    catalog = SchemaCatalog.load(ROOT / "spec/schema")
    schema_instances = 0
    schema_targets = sorted((ROOT / "fixtures").rglob("*.json")) + sorted((ROOT / "baselines").rglob("*.json"))
    for path in schema_targets:
        schema_id = None
        if path.name == "context.json":
            schema_id = "gluerift.adapter-context.v0.3.1a.schema.json"
        elif path.name == "scope.json":
            schema_id = "gluerift.validation-scope.v0.3.1a.schema.json"
        elif path.name == "policy.json":
            schema_id = "gluerift.policy.v0.3.1a.schema.json"
        elif path.name == "request.json":
            schema_id = "gluerift.validation-request.v0.3.1a.schema.json"
        elif path.name == "transformation.json":
            schema_id = "gluerift.transformation-candidate.v0.3.1a.schema.json"
        elif path.name == "composition.json":
            schema_id = "gluerift.composition-request.v0.3.1a.schema.json"
        elif path == REGISTRY_PATH:
            schema_id = "gluerift.fixture-registry.v0.3.1a.schema.json"
        elif path == ROOT / "fixtures/source-index.json":
            schema_id = "gluerift.fixture-source-index.v0.3.1a.schema.json"
        elif path.parent.parent == ROOT / "baselines" and path.name == "config.json":
            schema_id = "gluerift.baseline-configuration.v0.3.1a.schema.json"
        if schema_id is not None:
            catalog.validate(json.loads(path.read_bytes()), schema_id)
            schema_instances += 1
    assert schema_instances == len(json_files), "every fixture/baseline JSON must have an assigned schema"
    print(f"fixture declarations: proved-exhaustive ({len(rows)} runs, {len(json_files)} canonical JSON files)")
    print(f"fixture schema instances: proved-exhaustive ({schema_instances})")
    print(f"fixture registry sha256: {digest(registry)}")
    print(f"run configuration sha256: {run_config_hash}")
    print(f"transformation family sha256: {family_hash}")


if __name__ == "__main__":
    validate()
