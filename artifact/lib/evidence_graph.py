#!/usr/bin/env python3
"""Validate GlueRift's acyclic, layered canonical evidence graph."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path, PurePosixPath
from typing import Any, Callable

from jcs import canonical_sha256, load_json, sha256_bytes


EXPECTED_LAYERS = [
    "primary-inputs",
    "build-and-dependencies",
    "semantic-reference-evidence",
    "resolved-native-manifests",
    "native-replay-evidence",
    "claims",
    "result-owner",
    "paper-tables",
]


class EvidenceGraphError(ValueError):
    pass


def resolve_logical(root: Path, logical: str) -> Path:
    pure = PurePosixPath(logical)
    if pure.is_absolute() or not pure.parts or ".." in pure.parts:
        raise EvidenceGraphError(f"invalid repository-relative logical path: {logical}")
    path = root.joinpath(*pure.parts)
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise EvidenceGraphError(f"logical path escapes repository: {logical}") from error
    return path


def artifact_hash(path: Path, hash_mode: str) -> str:
    if hash_mode == "canonical-json":
        return canonical_sha256(load_json(path))
    if hash_mode == "raw-bytes":
        return sha256_bytes(path.read_bytes())
    raise EvidenceGraphError(f"unknown artifact hash mode: {hash_mode}")


def set_hash(entries: list[dict], predicate: Callable[[dict], bool] = lambda _: True) -> str:
    members = [
        {
            "evidence_id": entry["evidence_id"],
            "logical_path": entry["logical_path"],
            "sha256": entry["sha256"],
        }
        for entry in entries
        if predicate(entry)
    ]
    return canonical_sha256(members)


def require_content_hash(root: Path, logical: str, expected: str, *, raw: bool = False) -> dict | None:
    path = resolve_logical(root, logical)
    if not path.is_file():
        raise EvidenceGraphError(f"bound artifact is missing: {logical}")
    if raw:
        actual = sha256_bytes(path.read_bytes())
        document = None
    else:
        document = load_json(path)
        actual = canonical_sha256(document)
    if actual != expected:
        raise EvidenceGraphError(f"bound artifact hash mismatch: {logical}")
    return document


def collect_strings(value: Any) -> set[str]:
    if isinstance(value, str):
        return {value}
    if isinstance(value, list):
        output: set[str] = set()
        for item in value:
            output.update(collect_strings(item))
        return output
    if isinstance(value, dict):
        output = set()
        for key, item in value.items():
            output.add(key)
            output.update(collect_strings(item))
        return output
    return set()


def validate(root: Path, manifest_path: Path) -> dict[str, dict]:
    manifest = load_json(manifest_path)
    if manifest.get("schema") != "gluerift.reproduction-manifest/v0.3.1a":
        raise EvidenceGraphError("reproduction manifest schema mismatch")
    if manifest.get("semantic_contract_version") != "0.3.1a":
        raise EvidenceGraphError("reproduction manifest semantic version mismatch")
    if manifest_path.resolve() != (root / "artifact/reproduction-manifest.json").resolve():
        raise EvidenceGraphError("reproduction manifest must occupy its fixed canonical path")
    layers = manifest.get("layers")
    if not isinstance(layers, list) or [layer.get("layer_id") for layer in layers] != EXPECTED_LAYERS:
        raise EvidenceGraphError("reproduction layers are absent, reordered, or renamed")

    entries_by_path: dict[str, dict] = {}
    entries_by_evidence_id: dict[str, dict] = {}
    layer_entries: list[list[dict]] = []
    for layer_index, layer in enumerate(layers):
        entries = layer.get("entries")
        if not isinstance(entries, list):
            raise EvidenceGraphError(f"layer {layer['layer_id']} entries must be an array")
        if entries != sorted(entries, key=lambda entry: entry["logical_path"]):
            raise EvidenceGraphError(f"layer {layer['layer_id']} entries are not path-sorted")
        layer_entries.append(entries)
        for entry in entries:
            logical = entry.get("logical_path")
            if logical == "artifact/reproduction-manifest.json":
                raise EvidenceGraphError("root reproduction manifest cannot hash itself")
            if logical in entries_by_path:
                raise EvidenceGraphError(f"artifact appears in multiple graph layers: {logical}")
            path = resolve_logical(root, logical)
            if not path.is_file():
                raise EvidenceGraphError(f"graph artifact is missing: {logical}")
            actual = artifact_hash(path, entry.get("hash_mode"))
            if actual != entry.get("sha256"):
                raise EvidenceGraphError(f"graph artifact hash mismatch: {logical}")
            normalized = dict(entry)
            normalized["layer_index"] = layer_index
            entries_by_path[logical] = normalized
            evidence_id = entry.get("evidence_id")
            if evidence_id != "not-applicable":
                if not isinstance(evidence_id, str) or not evidence_id:
                    raise EvidenceGraphError(f"invalid evidence ID for {logical}")
                if evidence_id in entries_by_evidence_id:
                    raise EvidenceGraphError(f"duplicate evidence ID: {evidence_id}")
                entries_by_evidence_id[evidence_id] = normalized
                if entry.get("hash_mode") == "canonical-json":
                    document = load_json(path)
                    if document.get("evidence_id") != evidence_id:
                        raise EvidenceGraphError(f"internal evidence ID mismatch: {logical}")

    for logical, entry in entries_by_path.items():
        dependencies = entry.get("dependency_evidence_ids", [])
        if dependencies != sorted(set(dependencies)):
            raise EvidenceGraphError(f"dependency IDs must be sorted and duplicate-free: {logical}")
        for dependency in dependencies:
            target = entries_by_evidence_id.get(dependency)
            if target is None:
                raise EvidenceGraphError(f"unresolved dependency {dependency} from {logical}")
            if target["layer_index"] > entry["layer_index"]:
                raise EvidenceGraphError(f"non-topological dependency {logical} -> {dependency}")
        if entry.get("hash_mode") == "canonical-json":
            document = load_json(resolve_logical(root, logical))
            internal = document.get("dependency_evidence_ids")
            if internal is not None and internal != dependencies:
                raise EvidenceGraphError(f"internal dependency list mismatch: {logical}")

    # Coarse release layers deliberately group reports that can depend on one
    # another (for example, a transformation report depends on its base and
    # transformed checks).  Validate those same-layer edges as a real DAG
    # instead of pretending that path order is proof order.
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(evidence_id: str) -> None:
        if evidence_id in visited:
            return
        if evidence_id in visiting:
            raise EvidenceGraphError(f"evidence dependency cycle at {evidence_id}")
        visiting.add(evidence_id)
        entry = entries_by_evidence_id[evidence_id]
        for dependency in entry.get("dependency_evidence_ids", []):
            visit(dependency)
        visiting.remove(evidence_id)
        visited.add(evidence_id)

    for evidence_id in sorted(entries_by_evidence_id):
        visit(evidence_id)

    primary_expected = {
        "artifact/source-inputs.manifest.json",
        "fixtures/registry.json",
        "native/host-toolchain.lock.json",
        "spec/run-config/core-v0.3.1a.json",
        "spec/transformation-families/core-structural-v0.3.1a.json",
    }
    if {item["logical_path"] for item in layer_entries[0]} != primary_expected:
        raise EvidenceGraphError("primary layer differs from the five frozen Core inputs")
    if {item["logical_path"] for item in layer_entries[5]} != {"artifact/claims.json"}:
        raise EvidenceGraphError("claim layer does not have its unique canonical owner")
    if {item["logical_path"] for item in layer_entries[6]} != {"artifact/results/results.json"}:
        raise EvidenceGraphError("result layer does not have its unique canonical owner")
    if not layer_entries[7]:
        raise EvidenceGraphError("paper-table layer is empty")

    source_manifest = require_content_hash(
        root,
        "artifact/source-inputs.manifest.json",
        manifest["source_inputs_manifest_sha256"],
    )
    if source_manifest["source_tree_sha256"] != manifest["source_tree_sha256"]:
        raise EvidenceGraphError("source tree root binding differs from source manifest")
    require_content_hash(
        root, "spec/run-config/core-v0.3.1a.json", manifest["run_configuration_sha256"]
    )
    require_content_hash(
        root,
        "spec/transformation-families/core-structural-v0.3.1a.json",
        manifest["transformation_family_sha256"],
    )
    require_content_hash(root, "fixtures/registry.json", manifest["fixture_registry_sha256"])
    image_lock = require_content_hash(
        root, "native/host-toolchain.lock.json", manifest["pinned_host_toolchain_lock_sha256"]
    )
    if image_lock["host_toolchain_descriptor_sha256"] != manifest["pinned_host_toolchain_descriptor_sha256"]:
        raise EvidenceGraphError("pinned host/toolchain descriptor differs from its lock")

    build_schema = "gluerift.build-manifest/v0.3.1a"
    dynamic_schema = "gluerift.dynamic-dependency-manifest/v0.3.1a"
    build_entries = [
        entry
        for entry in layer_entries[1]
        if entry["logical_path"].endswith(".json")
        and load_json(resolve_logical(root, entry["logical_path"])).get("schema") == build_schema
    ]
    dynamic_entries = [
        entry
        for entry in layer_entries[1]
        if entry["logical_path"].endswith(".json")
        and load_json(resolve_logical(root, entry["logical_path"])).get("schema") == dynamic_schema
    ]
    if len(build_entries) != 3 or len(dynamic_entries) != 3:
        raise EvidenceGraphError("build layer must contain three build and dependency manifests")
    if set_hash(layer_entries[1], lambda item: item in build_entries) != manifest["build_manifest_set_sha256"]:
        raise EvidenceGraphError("root build-manifest set hash mismatch")
    if set_hash(layer_entries[1], lambda item: item in dynamic_entries) != manifest["dynamic_dependency_manifest_set_sha256"]:
        raise EvidenceGraphError("root dynamic-dependency set hash mismatch")
    if set_hash(layer_entries[3]) != manifest["resolved_native_manifest_set_sha256"]:
        raise EvidenceGraphError("root resolved-native-manifest set hash mismatch")
    combined_evidence = sorted(
        [*layer_entries[2], *layer_entries[4]], key=lambda item: item["logical_path"]
    )
    if set_hash(combined_evidence) != manifest["evidence_report_set_sha256"]:
        raise EvidenceGraphError("root evidence-report set hash mismatch")
    if set_hash(layer_entries[7]) != manifest["paper_table_set_sha256"]:
        raise EvidenceGraphError("root paper-table set hash mismatch")
    require_content_hash(root, "artifact/claims.json", manifest["claim_manifest_sha256"])
    results = require_content_hash(
        root, "artifact/results/results.json", manifest["result_owner_sha256"]
    )

    build_index = load_json(root / "artifact/evidence/native/build/index.json")
    if build_index.get("schema") != "gluerift.native-build-index/v0.3.1a":
        raise EvidenceGraphError("native build index is absent from the build layer")
    provisioning = require_content_hash(
        root,
        build_index["dependency_cache_provisioning_logical_path"],
        build_index["dependency_cache_provisioning_sha256"],
    )
    if provisioning.get("network_mode") != "disabled":
        raise EvidenceGraphError("dependency provisioning is not network-disabled")
    expected_roles = ["go-source", "native-harness", "rust-target"]
    if [item["role"] for item in build_index["entries"]] != expected_roles:
        raise EvidenceGraphError("native build index roles are incomplete or unordered")
    role_build_hashes: list[dict] = []
    role_dynamic_hashes: list[dict] = []
    for item in build_index["entries"]:
        build_document = require_content_hash(
            root, item["build_manifest_logical_path"], item["build_manifest_sha256"]
        )
        dependency_document = require_content_hash(
            root,
            item["dynamic_dependency_manifest_logical_path"],
            item["dynamic_dependency_manifest_sha256"],
        )
        if build_document.get("schema") != build_schema or dependency_document.get("schema") != dynamic_schema:
            raise EvidenceGraphError(f"{item['role']}: build index resolves the wrong schema")
        for key, expected in {
            "host_toolchain_descriptor_sha256": manifest["pinned_host_toolchain_descriptor_sha256"],
            "dependency_cache_provisioning_sha256": build_index[
                "dependency_cache_provisioning_sha256"
            ],
            "dynamic_dependency_manifest_sha256": item[
                "dynamic_dependency_manifest_sha256"
            ],
            "source_inputs_manifest_sha256": manifest["source_inputs_manifest_sha256"],
            "source_tree_sha256": manifest["source_tree_sha256"],
        }.items():
            if build_document.get(key) != expected:
                raise EvidenceGraphError(f"{item['role']}: build binding mismatch for {key}")
        for mapping_name in ("source_file_hashes", "lockfile_hashes"):
            for logical, digest in build_document[mapping_name].items():
                require_content_hash(root, logical, digest, raw=True)
        role_build_hashes.append(
            {"build_manifest_sha256": item["build_manifest_sha256"], "role": item["role"]}
        )
        role_dynamic_hashes.append(
            {
                "dynamic_dependency_manifest_sha256": item[
                    "dynamic_dependency_manifest_sha256"
                ],
                "role": item["role"],
            }
        )
    native_build_set = canonical_sha256(role_build_hashes)
    native_dynamic_set = canonical_sha256(role_dynamic_hashes)

    native_documents = [
        load_json(resolve_logical(root, item["logical_path"])) for item in layer_entries[3]
    ]
    if sorted(item.get("fixture_id") for item in native_documents) != ["E01", "E02"] or any(
        item.get("schema") != "gluerift.native-manifest/v0.3.1a" for item in native_documents
    ):
        raise EvidenceGraphError("resolved native layer must contain exactly E01 and E02 manifests")
    for document in native_documents:
        if document["host_toolchain_descriptor_sha256"] != manifest["pinned_host_toolchain_descriptor_sha256"]:
            raise EvidenceGraphError("native manifest host/toolchain binding mismatch")
        if document["build_manifest_set_sha256"] != native_build_set:
            raise EvidenceGraphError("native manifest build set binding mismatch")
        if document["dynamic_dependency_manifest_set_sha256"] != native_dynamic_set:
            raise EvidenceGraphError("native manifest dependency set binding mismatch")

    native_index = load_json(root / "artifact/evidence/native/index.json")
    if native_index.get("schema") != "gluerift.native-output-index/v0.3.1a" or [
        item.get("fixture_id") for item in native_index.get("fixtures", [])
    ] != ["E01", "E02"]:
        raise EvidenceGraphError("native output index is incomplete or unordered")
    for item in native_index["fixtures"]:
        for prefix, raw in (
            ("backend_conformance", False),
            ("native_manifest", False),
            ("reference_bundle", False),
            ("replay_report", False),
            ("transcript", True),
        ):
            require_content_hash(
                root, item[f"{prefix}_logical_path"], item[f"{prefix}_sha256"], raw=raw
            )

    expected_result_index = sorted(
        (
            {
                "evidence_id": evidence_id,
                "logical_path": item["logical_path"],
                "profile": item["profile"],
                "schema": load_json(resolve_logical(root, item["logical_path"]))["schema"],
                "sha256": item["sha256"],
            }
            for evidence_id, item in entries_by_evidence_id.items()
            if item["layer_index"] in {2, 4}
        ),
        key=lambda item: item["evidence_id"],
    )
    if results.get("evidence_index") != expected_result_index:
        raise EvidenceGraphError("result owner evidence index differs from graph evidence owners")

    for logical, entry in entries_by_path.items():
        if entry.get("hash_mode") != "canonical-json":
            continue
        document = load_json(resolve_logical(root, logical))
        if document.get("schema") != "gluerift.claim-manifest/v0.3.1a":
            continue
        for claim in document.get("claims", []):
            for required in claim.get("required_evidence_ids", []):
                target = entries_by_evidence_id.get(required)
                if target is None:
                    raise EvidenceGraphError(
                        f"claim {claim.get('claim_id')} has unresolved evidence {required}"
                    )
                if target["layer_index"] >= entry["layer_index"]:
                    raise EvidenceGraphError(
                        f"claim {claim.get('claim_id')} references non-prior evidence {required}"
                    )

    # No document in a lower layer may contain the content hash of a later layer.
    for layer_index, entries in enumerate(layer_entries):
        later_hashes = {
            entry["sha256"]
            for later in layer_entries[layer_index + 1 :]
            for entry in later
        }
        for entry in entries:
            if entry.get("hash_mode") != "canonical-json":
                continue
            strings = collect_strings(load_json(resolve_logical(root, entry["logical_path"])))
            leaked = sorted(strings & later_hashes)
            if leaked:
                raise EvidenceGraphError(
                    f"lower graph layer {entry['logical_path']} contains later-layer hash {leaked[0]}"
                )

    required_top_level = {
        "build_manifest_set_sha256",
        "claim_manifest_sha256",
        "dynamic_dependency_manifest_set_sha256",
        "evidence_report_set_sha256",
        "source_inputs_manifest_sha256",
        "source_tree_sha256",
        "run_configuration_sha256",
        "transformation_family_sha256",
        "fixture_registry_sha256",
        "paper_table_set_sha256",
        "pinned_host_toolchain_descriptor_sha256",
        "pinned_host_toolchain_lock_sha256",
        "resolved_native_manifest_set_sha256",
        "result_owner_sha256",
    }
    missing = sorted(required_top_level - set(manifest))
    if missing:
        raise EvidenceGraphError(f"reproduction manifest lacks root bindings: {missing}")
    return entries_by_path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    entries = validate(args.root.resolve(), args.manifest)
    print(f"evidence graph: proved-exhaustive ({len(entries)} artifacts)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, EvidenceGraphError) as error:
        print(f"evidence graph error: {error}", file=sys.stderr)
        raise SystemExit(4)
