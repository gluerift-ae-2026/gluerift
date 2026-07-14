#!/usr/bin/env python3
"""Deterministic validator for the JSON-Schema subset used by GlueRift Core."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from jcs import canonical_bytes, load_json


ALLOWED_KEYWORDS = {
    "$defs",
    "$id",
    "$ref",
    "$schema",
    "additionalProperties",
    "allOf",
    "const",
    "description",
    "enum",
    "items",
    "maxItems",
    "maxLength",
    "maxProperties",
    "maximum",
    "minItems",
    "minLength",
    "minProperties",
    "minimum",
    "oneOf",
    "pattern",
    "properties",
    "required",
    "title",
    "type",
    "unevaluatedProperties",
    "uniqueItems",
}

CONTEXTUAL_SCHEMA_IDS = {
    "composition.json": "gluerift.composition-request.v0.3.1a.schema.json",
    "transformation.json": "gluerift.transformation-candidate.v0.3.1a.schema.json",
}

ENVELOPE_KEYS = {
    "candidate_sha256", "comparator_spec_sha256", "dependency_evidence_ids",
    "endpoint_policy_sha256", "evidence_id", "run_configuration_sha256", "schema",
    "semantic_contract_version", "status", "tool_build_sha256", "types_sha256",
    "validation_request_sha256", "validation_scope_sha256",
}

# The JSON Schema engine deliberately implements a small deterministic subset.
# These exact-key contracts close the otherwise-open GenericReport wrappers and
# are part of the normative schema check, not a test-only assertion.
REPORT_KEYS: dict[str, tuple[set[str], ...]] = {
    "gluerift.check-report/v0.3.1a": (ENVELOPE_KEYS | {
        "bridges", "carrier_summary_sha256", "certification", "comparison", "policy",
        "properties", "roundtrip_report_sha256", "witness_sha256s",
    },),
    "gluerift.roundtrip-report/v0.3.1a": (ENVELOPE_KEYS | {"laws"},),
    "gluerift.bridge-report/v0.3.1a": (ENVELOPE_KEYS | {
        "bridge_kind", "carrier_comparator_evidence", "checked_pair_count",
        "counterexample_pair", "native_comparator_evidence", "sufficient_rule_coverage",
        "universe_pair_count",
    },),
    "gluerift.carrier-summary/v0.3.1a": (ENVELOPE_KEYS | {
        "applicability_to_selected_comparator", "bridge_report_sha256",
        "class_endpoint_pairs", "class_observation_conflicts", "evidence_basis",
        "shared_carrier_classes", "source_successful_image", "target_successful_image",
    },),
    "gluerift.execution-trace-table/v0.3.1a": (ENVELOPE_KEYS | {"law_id", "rows"},),
    "gluerift.derivation-report/v0.3.1a": (ENVELOPE_KEYS | {
        "adapter_path", "children", "exhaustive_crosscheck_sha256", "input_domain_sha256",
        "judgment_kind", "observer_paths", "output_domain_sha256", "relation_bridge",
        "relation_kind",
    },),
    "gluerift.transformation-report/v0.3.1a": (ENVELOPE_KEYS | {
        "action_domain", "action_domain_sha256", "base_alignment_status",
        "base_check_report_sha256", "base_source_decode_sha256", "base_source_encode_sha256",
        "base_target_decode_sha256", "base_target_encode_sha256", "candidate_binding_status",
        "candidate_context_sha256", "classification", "comparator_definedness_status",
        "family_completeness_statement", "four_map_construction_status",
        "four_map_semantics_check_sha256", "generation_mode", "generation_ordinal",
        "generation_parent_path", "generation_rule_id", "harmful_witness_sha256",
        "inapplicability_reasons", "inverse_check_status", "inverse_ir", "inverse_sha256",
        "lawfulness_status", "requested_law_ids", "roundtrip_statuses",
        "selected_property_statuses", "transformation_family_sha256", "transformation_ir",
        "transformation_sha256", "transformed_bridge_report_sha256",
        "transformed_check_report_sha256", "transformed_context_sha256",
        "transformed_source_decode_sha256", "transformed_source_encode_sha256",
        "transformed_target_decode_sha256", "transformed_target_encode_sha256",
        "twist_construction", "twist_side", "well_typed_status",
    },),
    "gluerift.witness/v0.3.1a": (ENVELOPE_KEYS | {
        "adapter_path", "comparator_evidence", "comparator_kind", "coverage_mode",
        "match_membership", "match_pair_count", "replay_command", "roundtrip_trace",
        "safe_membership", "source_comparison_domain_sha256", "source_value",
        "target_comparison_domain_sha256", "target_value", "violated_or_missing_dimensions",
        "witness_kind",
    },),
    "gluerift.backend-conformance/v0.3.1a": (ENVELOPE_KEYS | {
        "adapter_value_mismatches", "build_manifest_set_sha256", "build_manifests",
        "checked_adapter_value_count", "checked_comparator_pair_count", "comparator_kind",
        "comparator_truth_table_mismatches", "context_sha256",
        "dynamic_dependency_manifest_set_sha256", "dynamic_dependency_manifests", "fixture_id",
        "native_source_tree_sha256", "native_target_tree_sha256", "reference_check_evidence_id",
        "reference_bundle_evidence_id", "reference_bundle_logical_path",
        "reference_bundle_sha256", "roundtrip_truth_table_mismatches",
        "runtime_environment_sha256", "stdin_or_fixture_logical_path",
        "stdin_or_fixture_sha256",
    },),
    "gluerift.native-replay-report/v0.3.1a": (ENVELOPE_KEYS | {
        "backend_conformance_evidence_id", "bridge_statuses", "build_manifest_set_sha256",
        "comparator_definedness", "comparator_kind", "host_toolchain_descriptor_sha256",
        "context_sha256", "dynamic_dependency_manifest_set_sha256", "fixture_id",
        "native_manifest_sha256", "ordinary_comparator_result", "processes", "property_statuses",
        "property_witnesses", "reference_candidate_binding_status", "reference_candidate_sha256",
        "reference_check_evidence_id", "reference_check_report_sha256", "reference_run_id",
        "reference_bundle_evidence_id", "reference_bundle_logical_path",
        "reference_bundle_sha256",
        "runtime_environment_sha256", "six_roundtrip_statuses", "source_program_output",
        "source_tree_read_only", "source_tree_read_only_enforcement",
        "stdin_or_fixture_logical_path", "stdin_or_fixture_sha256", "target_program_output",
        "transformation_report_sha256", "transported_source_as_target_native",
        "violation_witness",
    },),
    "gluerift.proof-audit/v0.3.1a": (ENVELOPE_KEYS | {
        "axiom_audit_log_sha256", "hygiene_status", "lean_executable_sha256", "lean_version",
        "mechanized_groups", "source_entries",
    },),
}

BL2_KEYS = ENVELOPE_KEYS | {"baseline_id", "law_statuses", "paired_check_report_sha256"}
BL4_KEYS = BL2_KEYS | {
    "comparator_definedness", "coverage_parity_status", "derivation_parity_status",
    "derivation_report_sha256", "match_anchor_coverage", "match_coverage",
    "match_coverage_status", "match_shape_compatibility", "match_subset_safe",
    "policy_contract_status", "policy_parity_status", "policy_witnesses",
    "profile_property_consistency", "property_parity_status", "property_statuses",
    "property_witnesses", "safe_anchor_coverage", "target_non_amplification",
    "validity_parity_status", "witness_parity_status",
}
REPORT_KEYS["gluerift.baseline-report/v0.3.1a"] = (BL2_KEYS, BL4_KEYS)


class SchemaError(ValueError):
    pass


@dataclass(frozen=True)
class SchemaCatalog:
    directory: Path
    documents: dict[str, Any]

    @classmethod
    def load(cls, directory: Path) -> "SchemaCatalog":
        documents: dict[str, Any] = {}
        for path in sorted(directory.glob("*.schema.json")):
            document = load_json(path)
            if not isinstance(document, dict):
                raise SchemaError(f"{path.name}: schema root must be an object")
            if document.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
                raise SchemaError(f"{path.name}: unsupported or missing $schema")
            identifier = document.get("$id")
            if identifier != path.name:
                raise SchemaError(f"{path.name}: $id must equal the logical filename")
            if identifier in documents:
                raise SchemaError(f"duplicate schema $id: {identifier}")
            documents[identifier] = document
        if not documents:
            raise SchemaError("schema catalog is empty")
        catalog = cls(directory=directory, documents=documents)
        for identifier, document in documents.items():
            catalog._audit_schema(document, identifier, "$", document)
        return catalog

    def _audit_schema(self, node: Any, document_id: str, path: str, root: Any) -> None:
        if isinstance(node, list):
            for index, child in enumerate(node):
                self._audit_schema(child, document_id, f"{path}[{index}]", root)
            return
        if not isinstance(node, dict):
            return
        unknown = sorted(set(node) - ALLOWED_KEYWORDS)
        if unknown:
            raise SchemaError(f"{document_id}:{path}: unsupported keywords {unknown}")
        if "$ref" in node:
            self.resolve(node["$ref"], document_id)
        if "required" in node:
            required = node["required"]
            if not isinstance(required, list) or len(required) != len(set(required)):
                raise SchemaError(f"{document_id}:{path}: required must be duplicate-free")
        if "type" in node:
            declared = node["type"]
            types = [declared] if isinstance(declared, str) else declared
            if not isinstance(types, list) or not set(types) <= {
                "array",
                "boolean",
                "integer",
                "null",
                "number",
                "object",
                "string",
            }:
                raise SchemaError(f"{document_id}:{path}: invalid type declaration")
        for key, child in node.items():
            if key in {"const", "enum", "required", "type", "$id", "$ref", "$schema"}:
                continue
            if key in {"$defs", "properties"}:
                if not isinstance(child, dict):
                    raise SchemaError(f"{document_id}:{path}.{key}: expected object")
                for name, subschema in child.items():
                    self._audit_schema(subschema, document_id, f"{path}.{key}.{name}", root)
                continue
            self._audit_schema(child, document_id, f"{path}.{key}", root)

    def resolve(self, reference: str, current_id: str) -> tuple[Any, str]:
        filename, marker, fragment = reference.partition("#")
        target_id = filename or current_id
        if target_id not in self.documents:
            raise SchemaError(f"{current_id}: unresolved schema reference {reference}")
        target: Any = self.documents[target_id]
        if marker and fragment:
            if not fragment.startswith("/"):
                raise SchemaError(f"{current_id}: only JSON Pointer fragments are supported")
            for escaped in fragment[1:].split("/"):
                token = escaped.replace("~1", "/").replace("~0", "~")
                if not isinstance(target, dict) or token not in target:
                    raise SchemaError(f"{current_id}: unresolved JSON Pointer {reference}")
                target = target[token]
        return target, target_id

    def validate(self, instance: Any, schema_id: str) -> None:
        if schema_id not in self.documents:
            raise SchemaError(f"unknown schema ID: {schema_id}")
        self._validate(instance, self.documents[schema_id], schema_id, "$")

    def _validate(self, value: Any, schema: Any, current_id: str, path: str) -> None:
        if schema is True:
            return
        if schema is False:
            raise SchemaError(f"{path}: rejected by false schema")
        if not isinstance(schema, dict):
            raise SchemaError(f"{current_id}:{path}: schema node is not an object")

        if "$ref" in schema:
            target, target_id = self.resolve(schema["$ref"], current_id)
            self._validate(value, target, target_id, path)
            siblings = {key: item for key, item in schema.items() if key not in {"$ref", "$id", "$schema"}}
            if siblings:
                self._validate(value, siblings, current_id, path)

        if "allOf" in schema:
            for child in schema["allOf"]:
                self._validate(value, child, current_id, path)
        if "oneOf" in schema:
            successes = 0
            errors = []
            for child in schema["oneOf"]:
                try:
                    self._validate(value, child, current_id, path)
                    successes += 1
                except SchemaError as error:
                    errors.append(str(error))
            if successes != 1:
                raise SchemaError(
                    f"{path}: expected exactly one matching alternative, got {successes}; "
                    f"first error: {errors[0] if errors else 'none'}"
                )
        if "const" in schema and value != schema["const"]:
            raise SchemaError(f"{path}: value does not equal required constant")
        if "enum" in schema and value not in schema["enum"]:
            raise SchemaError(f"{path}: value is not in the declared enumeration")
        if "type" in schema:
            declared = schema["type"]
            types = [declared] if isinstance(declared, str) else declared
            if not any(_is_type(value, expected) for expected in types):
                raise SchemaError(f"{path}: expected type {types}, got {_type_name(value)}")

        if isinstance(value, dict):
            required = schema.get("required", [])
            missing = [key for key in required if key not in value]
            if missing:
                raise SchemaError(f"{path}: missing required properties {missing}")
            properties = schema.get("properties", {})
            for key, child in properties.items():
                if key in value:
                    self._validate(value[key], child, current_id, f"{path}.{key}")
            extras = sorted(set(value) - set(properties))
            additional = schema.get("additionalProperties", True)
            if additional is False and extras:
                raise SchemaError(f"{path}: undeclared properties {extras}")
            if isinstance(additional, dict):
                for key in extras:
                    self._validate(value[key], additional, current_id, f"{path}.{key}")
            if len(value) < schema.get("minProperties", 0):
                raise SchemaError(f"{path}: too few object properties")
            if "maxProperties" in schema and len(value) > schema["maxProperties"]:
                raise SchemaError(f"{path}: too many object properties")

        if isinstance(value, list):
            if len(value) < schema.get("minItems", 0):
                raise SchemaError(f"{path}: too few array items")
            if "maxItems" in schema and len(value) > schema["maxItems"]:
                raise SchemaError(f"{path}: too many array items")
            if schema.get("uniqueItems"):
                encoded = [canonical_bytes(item) for item in value]
                if len(encoded) != len(set(encoded)):
                    raise SchemaError(f"{path}: duplicate array item")
            if "items" in schema:
                for index, item in enumerate(value):
                    self._validate(item, schema["items"], current_id, f"{path}[{index}]")

        if isinstance(value, str):
            if len(value) < schema.get("minLength", 0):
                raise SchemaError(f"{path}: string shorter than minLength")
            if "maxLength" in schema and len(value) > schema["maxLength"]:
                raise SchemaError(f"{path}: string longer than maxLength")
            if "pattern" in schema and re.search(schema["pattern"], value) is None:
                raise SchemaError(f"{path}: string does not match pattern {schema['pattern']}")

        if isinstance(value, (int, float)) and not isinstance(value, bool):
            if "minimum" in schema and value < schema["minimum"]:
                raise SchemaError(f"{path}: number below minimum")
            if "maximum" in schema and value > schema["maximum"]:
                raise SchemaError(f"{path}: number above maximum")


def _is_type(value: Any, expected: str) -> bool:
    return {
        "array": isinstance(value, list),
        "boolean": isinstance(value, bool),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "null": value is None,
        "number": isinstance(value, (int, float)) and not isinstance(value, bool),
        "object": isinstance(value, dict),
        "string": isinstance(value, str),
    }[expected]


def _type_name(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int):
        return "integer"
    if isinstance(value, float):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, list):
        return "array"
    if isinstance(value, dict):
        return "object"
    return type(value).__name__


def _exact_keys(document: dict[str, Any], alternatives: tuple[set[str], ...], path: Path) -> None:
    actual = set(document)
    if actual not in alternatives:
        candidates = sorted(alternatives, key=lambda item: len(actual ^ item))
        expected = candidates[0]
        raise SchemaError(
            f"{path}: semantic schema key mismatch; missing={sorted(expected - actual)}, "
            f"extra={sorted(actual - expected)}"
        )


def validate_semantics(document: dict[str, Any], path: Path) -> None:
    schema = document.get("schema")
    alternatives = REPORT_KEYS.get(schema)
    if alternatives is not None:
        _exact_keys(document, alternatives, path)
        dependencies = document["dependency_evidence_ids"]
        if dependencies != sorted(set(dependencies)):
            raise SchemaError(f"{path}: evidence dependencies are not canonical")

    if schema == "gluerift.transformation-report/v0.3.1a":
        if canonical_bytes(document["action_domain"]):
            from jcs import canonical_sha256

            for key, value in (
                ("action_domain_sha256", document["action_domain"]),
                ("transformation_sha256", document["transformation_ir"]),
                ("inverse_sha256", document["inverse_ir"]),
            ):
                if document[key] != canonical_sha256(value):
                    raise SchemaError(f"{path}: transformation content binding mismatch: {key}")
        law_keys = {
            "source-native", "target-native", "source-carrier", "target-carrier",
            "source-full-transport", "target-full-transport",
        }
        if set(document["roundtrip_statuses"]) != law_keys:
            raise SchemaError(f"{path}: transformation report lacks the six law results")
        all_lawful = set(document["roundtrip_statuses"].values()) == {"proved-exhaustive"}
        all_structural = all(
            document[key] == "proved-exhaustive"
            for key in (
                "well_typed_status", "inverse_check_status", "four_map_construction_status",
                "comparator_definedness_status",
            )
        )
        expected_lawfulness = "proved-exhaustive" if all_lawful and all_structural else "disproved"
        if document["lawfulness_status"] != expected_lawfulness:
            raise SchemaError(f"{path}: lawfulness status is inconsistent with its premises")
        if not (all_lawful and all_structural) and document["classification"] != "law-breaking-or-inapplicable":
            raise SchemaError(f"{path}: inapplicable candidate entered a lawful bucket")

    if schema == "gluerift.check-report/v0.3.1a":
        certification = document["certification"]
        policy = document["policy"]
        if policy["policy_contract_status"] == "policy-unconstrained" and certification["granted"]:
            raise SchemaError(f"{path}: unconstrained policy received a security certificate")
        if policy["safe_dimension_count"] == 0 and not policy["policy_vacuity_warning"]:
            raise SchemaError(f"{path}: empty Safe dimensions lack a vacuity warning")
        if certification["granted"] and certification["blocking_reasons"]:
            raise SchemaError(f"{path}: granted certificate retains blocking reasons")

    if schema == "gluerift.native-output-index/v0.3.1a":
        expected = {
            "backend_conformance_logical_path", "backend_conformance_sha256", "fixture_id",
            "native_manifest_logical_path", "native_manifest_sha256", "replay_report_logical_path",
            "reference_bundle_logical_path", "reference_bundle_sha256",
            "replay_report_sha256", "transcript_logical_path", "transcript_sha256",
        }
        if [item.get("fixture_id") for item in document["fixtures"]] != ["E01", "E02"]:
            raise SchemaError(f"{path}: native index must be ordered E01, E02")
        for item in document["fixtures"]:
            if set(item) != expected:
                raise SchemaError(f"{path}: native index entry has an open or incomplete shape")

    if schema == "gluerift.native-manifest/v0.3.1a":
        if [item.get("role") for item in document["executables"]] != [
            "go-source", "native-harness", "rust-target"
        ]:
            raise SchemaError(f"{path}: native executable roles are incomplete or unordered")

    if schema == "gluerift.fixture-results/v0.3.1a":
        if len(document["runs"]) != 24:
            raise SchemaError(f"{path}: fixture aggregate must contain the 24 Core rows")
        ids = [item.get("run_id") for item in document["runs"]]
        if ids != sorted(set(ids)):
            raise SchemaError(f"{path}: fixture rows are not unique and ordered")

    if schema == "gluerift.results/v0.3.1a":
        _exact_keys(
            document,
            ({"schema", "semantic_contract_version", "runs", "evidence_index", "native_replays", "proof_audit"},),
            path,
        )
        if len(document["runs"]) != 24 or not document["evidence_index"]:
            raise SchemaError(f"{path}: result owner is incomplete")
        ids = [item.get("evidence_id") for item in document["evidence_index"]]
        if ids != sorted(set(ids)):
            raise SchemaError(f"{path}: result evidence index is not canonical")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--schema-dir", type=Path, required=True)
    parser.add_argument("--instance", type=Path)
    parser.add_argument("--schema-id")
    parser.add_argument("--instances", type=Path, action="append")
    args = parser.parse_args()
    catalog = SchemaCatalog.load(args.schema_dir)
    if bool(args.instance) != bool(args.schema_id):
        raise SchemaError("--instance and --schema-id must be supplied together")
    if args.instance:
        document = load_json(args.instance)
        catalog.validate(document, args.schema_id)
        if isinstance(document, dict):
            validate_semantics(document, args.instance)
    validated_instances = 0
    for root in args.instances or []:
        paths = sorted(root.rglob("*.json")) if root.is_dir() else [root]
        for path in paths:
            document = load_json(path)
            if not isinstance(document, dict):
                raise SchemaError(f"{path}: canonical JSON root is not an object")
            if isinstance(document.get("schema"), str):
                schema_id = document["schema"].replace("/", ".") + ".schema.json"
            else:
                schema_id = CONTEXTUAL_SCHEMA_IDS.get(path.name, "")
                if not schema_id:
                    raise SchemaError(f"{path}: canonical JSON object has no schema discriminator")
            if schema_id not in catalog.documents:
                raise SchemaError(f"{path}: no schema document {schema_id}")
            catalog.validate(document, schema_id)
            validate_semantics(document, path)
            validated_instances += 1
    print(f"schema catalog: proved-exhaustive ({len(catalog.documents)} documents)")
    if validated_instances:
        print(f"schema instances: proved-exhaustive ({validated_instances} documents)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, SchemaError, json.JSONDecodeError) as error:
        print(f"schema error: {error}", file=sys.stderr)
        raise SystemExit(4)
