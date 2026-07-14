#!/usr/bin/env python3
"""Construct the fixed, acyclic GlueRift reproduction evidence graph."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path, PurePosixPath
from typing import Iterable

from jcs import canonical_sha256, load_json, sha256_bytes, write_canonical


class ManifestError(ValueError):
    pass


LAYER_IDS = (
    "primary-inputs",
    "build-and-dependencies",
    "semantic-reference-evidence",
    "resolved-native-manifests",
    "native-replay-evidence",
    "claims",
    "result-owner",
    "paper-tables",
)


def _logical(root: Path, path: Path) -> str:
    try:
        relative = path.resolve().relative_to(root.resolve())
    except ValueError as error:
        raise ManifestError(f"artifact is outside release root: {path}") from error
    logical = PurePosixPath(*relative.parts).as_posix()
    if not logical or logical.startswith("/") or ".." in PurePosixPath(logical).parts:
        raise ManifestError(f"invalid logical path: {logical}")
    return logical


def _entry(root: Path, path: Path, profile: str = "core") -> dict:
    if not path.is_file():
        raise ManifestError(f"graph artifact does not exist: {path}")
    logical = _logical(root, path)
    if path.suffix == ".json":
        document = load_json(path)
        return {
            "dependency_evidence_ids": sorted(document.get("dependency_evidence_ids", [])),
            "evidence_id": document.get("evidence_id", "not-applicable"),
            "hash_mode": "canonical-json",
            "logical_path": logical,
            "profile": profile,
            "sha256": canonical_sha256(document),
        }
    return {
        "dependency_evidence_ids": [],
        "evidence_id": "not-applicable",
        "hash_mode": "raw-bytes",
        "logical_path": logical,
        "profile": profile,
        "sha256": sha256_bytes(path.read_bytes()),
    }


def _files(paths: Iterable[Path]) -> list[Path]:
    output: set[Path] = set()
    for path in paths:
        if path.is_dir():
            output.update(item for item in path.rglob("*") if item.is_file())
        elif path.is_file():
            output.add(path)
        else:
            raise ManifestError(f"graph input is missing: {path}")
    return sorted(output, key=lambda item: item.as_posix())


def _layer(root: Path, layer_id: str, files: Iterable[Path]) -> dict:
    entries = [_entry(root, path) for path in files]
    entries.sort(key=lambda item: item["logical_path"])
    if len({item["logical_path"] for item in entries}) != len(entries):
        raise ManifestError(f"duplicate artifact in layer {layer_id}")
    return {"entries": entries, "layer_id": layer_id}


def _partition_generated(
    paths: list[Path],
) -> tuple[list[Path], list[Path], list[Path], list[Path]]:
    build: list[Path] = []
    native_manifests: list[Path] = []
    semantic_reference: list[Path] = []
    native_replay: list[Path] = []
    for path in paths:
        if path.suffix != ".json":
            if "/artifact/evidence/native/" in path.as_posix():
                native_replay.append(path)
            else:
                semantic_reference.append(path)
            continue
        schema = load_json(path).get("schema")
        if schema in {
            "gluerift.build-manifest/v0.3.1a",
            "gluerift.dependency-cache-provisioning/v0.3.1a",
            "gluerift.dynamic-dependency-manifest/v0.3.1a",
            "gluerift.native-build-index/v0.3.1a",
        }:
            build.append(path)
        elif path.name == "constructed-context.json" and schema == "gluerift.adapter-context/v0.3.1a":
            # Generated contexts are dependency objects for both the resolved
            # native manifests and later semantic reports.  They therefore
            # precede the native-manifest layer, but are excluded from the
            # role-indexed build-manifest set hash below.
            build.append(path)
        elif schema == "gluerift.native-manifest/v0.3.1a":
            native_manifests.append(path)
        elif "/artifact/evidence/native/" in path.as_posix():
            native_replay.append(path)
        else:
            semantic_reference.append(path)
    return build, semantic_reference, native_manifests, native_replay


def build(args: argparse.Namespace) -> dict:
    root = args.root.resolve()
    source_manifest_path = args.source_manifest.resolve()
    source_manifest = load_json(source_manifest_path)
    image_lock = load_json(args.image_lock)
    if image_lock["host_toolchain_descriptor_sha256"] != args.pinned_host_toolchain_descriptor_sha256:
        raise ManifestError("pinned host/toolchain descriptor does not match its lock")
    primary_paths = [
        source_manifest_path,
        args.run_configuration.resolve(),
        args.transformation_family.resolve(),
        args.fixture_registry.resolve(),
        args.image_lock.resolve(),
    ]
    generated = _files(args.generated)
    build_paths, semantic_paths, native_paths, native_replay_paths = _partition_generated(
        generated
    )

    reserved = {
        args.claims.resolve(),
        args.results.resolve(),
        args.out.resolve(),
        *[item.resolve() for item in _files(args.tables)],
    }
    semantic_paths = [item for item in semantic_paths if item.resolve() not in reserved]
    native_replay_paths = [
        item for item in native_replay_paths if item.resolve() not in reserved
    ]
    build_paths = [item for item in build_paths if item.resolve() not in reserved]
    native_paths = [item for item in native_paths if item.resolve() not in reserved]

    layers = [
        _layer(root, LAYER_IDS[0], primary_paths),
        _layer(root, LAYER_IDS[1], build_paths),
        _layer(root, LAYER_IDS[2], semantic_paths),
        _layer(root, LAYER_IDS[3], native_paths),
        _layer(root, LAYER_IDS[4], native_replay_paths),
        _layer(root, LAYER_IDS[5], [args.claims.resolve()]),
        _layer(root, LAYER_IDS[6], [args.results.resolve()]),
        _layer(root, LAYER_IDS[7], _files(args.tables)),
    ]
    all_paths = [entry["logical_path"] for layer in layers for entry in layer["entries"]]
    if len(all_paths) != len(set(all_paths)):
        raise ManifestError("an artifact appears in more than one graph layer")

    def set_hash(layer_index: int, predicate=lambda entry: True) -> str:
        members = [
            {
                "evidence_id": entry["evidence_id"],
                "logical_path": entry["logical_path"],
                "sha256": entry["sha256"],
            }
            for entry in layers[layer_index]["entries"]
            if predicate(entry)
        ]
        return canonical_sha256(members)

    def combined_set_hash(layer_indices: tuple[int, ...]) -> str:
        members = [
            {
                "evidence_id": entry["evidence_id"],
                "logical_path": entry["logical_path"],
                "sha256": entry["sha256"],
            }
            for layer_index in layer_indices
            for entry in layers[layer_index]["entries"]
        ]
        members.sort(key=lambda item: item["logical_path"])
        return canonical_sha256(members)

    build_manifest_paths = {
        _logical(root, path)
        for path in build_paths
        if path.suffix == ".json"
        and load_json(path).get("schema") == "gluerift.build-manifest/v0.3.1a"
    }
    dynamic_dependency_paths = {
        _logical(root, path)
        for path in build_paths
        if path.suffix == ".json"
        and load_json(path).get("schema")
        == "gluerift.dynamic-dependency-manifest/v0.3.1a"
    }

    return {
        "build_manifest_set_sha256": set_hash(
            1, lambda entry: entry["logical_path"] in build_manifest_paths
        ),
        "claim_manifest_sha256": canonical_sha256(load_json(args.claims)),
        "dynamic_dependency_manifest_set_sha256": set_hash(
            1, lambda entry: entry["logical_path"] in dynamic_dependency_paths
        ),
        "evidence_report_set_sha256": combined_set_hash((2, 4)),
        "fixture_registry_sha256": canonical_sha256(load_json(args.fixture_registry)),
        "pinned_host_toolchain_descriptor_sha256": image_lock["host_toolchain_descriptor_sha256"],
        "pinned_host_toolchain_lock_sha256": canonical_sha256(image_lock),
        "paper_table_set_sha256": set_hash(7),
        "resolved_native_manifest_set_sha256": set_hash(3),
        "result_owner_sha256": canonical_sha256(load_json(args.results)),
        "layers": layers,
        "run_configuration_sha256": canonical_sha256(load_json(args.run_configuration)),
        "schema": "gluerift.reproduction-manifest/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "source_inputs_manifest_sha256": canonical_sha256(source_manifest),
        "source_tree_sha256": source_manifest["source_tree_sha256"],
        "transformation_family_sha256": canonical_sha256(load_json(args.transformation_family)),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--source-manifest", type=Path, required=True)
    parser.add_argument("--run-configuration", type=Path, required=True)
    parser.add_argument("--transformation-family", type=Path, required=True)
    parser.add_argument("--fixture-registry", type=Path, required=True)
    parser.add_argument("--image-lock", type=Path, required=True)
    parser.add_argument("--generated", type=Path, action="append", required=True)
    parser.add_argument("--claims", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--tables", type=Path, action="append", required=True)
    parser.add_argument("--pinned-host-toolchain-descriptor-sha256", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    document = build(args)
    write_canonical(args.out, document)
    print(sum(len(layer["entries"]) for layer in document["layers"]))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ManifestError, TypeError) as error:
        print(f"reproduction-manifest error: {error}", file=sys.stderr)
        raise SystemExit(4)
