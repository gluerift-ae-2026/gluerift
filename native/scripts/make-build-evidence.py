#!/usr/bin/env python3
"""Create content-bound native build and dependency manifests.

The output intentionally contains no host checkout path. Actual host paths are
noncanonical invocation telemetry; canonical tool paths live in the bound OS
image namespace and are associated with the exact executable digests here.
"""

import argparse
import hashlib
import json
import os
import pathlib
import platform
import subprocess
import sys


SCHEMA_VERSION = "v0.3.1a"
RUNTIME_BASE_ENV = {
    "LANG": "C.UTF-8",
    "LC_ALL": "C.UTF-8",
    "SOURCE_DATE_EPOCH": "1767225600",
    "TZ": "UTC",
}


def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()


def digest_bytes(value):
    return hashlib.sha256(value).hexdigest()


def digest_file(path):
    with open(path, "rb") as handle:
        return digest_bytes(handle.read())


def command(*argv):
    completed = subprocess.run(argv, check=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env={
        "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"
    })
    return completed.stdout.decode(errors="strict").strip()


def source_hashes(repo, roots):
    result = {}
    for root in roots:
        for path in sorted((repo / root).rglob("*")):
            if path.is_file() and "target" not in path.parts:
                logical = path.relative_to(repo).as_posix()
                result[logical] = digest_file(path)
    return result


def dependencies(binary, descriptor_sha256):
    output = command("/usr/bin/otool", "-L", str(binary)).splitlines()
    libraries = []
    for line in output:
        identity = line.strip()
        if not identity or identity == f"{binary}:":
            continue
        path, separator, _version = identity.partition(" (compatibility version")
        if not separator:
            raise ValueError(f"unexpected otool -L output for {binary}: {identity}")
        actual = pathlib.Path(path)
        if actual.is_file():
            content_sha = digest_file(actual)
            basis = "file-bytes"
        else:
            # Modern Darwin ships system libraries in the dyld shared cache.
            # Bind dyld's parsed image description to the pinned host/toolchain
            # descriptor instead of pretending an ordinary file path exists.
            parsed = command("/usr/bin/dyld_info", "-segments", path)
            content_sha = digest_bytes(canonical({
                "dyld_image_description": parsed,
                "host_toolchain_descriptor_sha256": descriptor_sha256,
            }))
            basis = "dyld-image-description-plus-host-toolchain-descriptor"
        libraries.append({
            "hash_basis": basis,
            "image_internal_path": path,
            "library_identity": identity,
            "sha256": content_sha,
        })
    libraries.sort(key=lambda value: (value["image_internal_path"], value["library_identity"]))
    return {
        "host_toolchain_descriptor_sha256": descriptor_sha256,
        "libraries": libraries,
        "schema": f"gluerift.dynamic-dependency-manifest/{SCHEMA_VERSION}",
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bindings", required=True)
    parser.add_argument("--cc", required=True)
    parser.add_argument("--cargo", required=True)
    parser.add_argument("--go", required=True)
    parser.add_argument("--go-source", required=True)
    parser.add_argument("--linker", required=True)
    parser.add_argument("--native-harness", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--rust-target", required=True)
    parser.add_argument("--rustc", required=True)
    args = parser.parse_args()
    repo = pathlib.Path(args.repo).resolve()
    with open(args.bindings, "rb") as handle:
        bindings = json.load(handle)
    source_tree_sha = bindings["source_tree_sha256"]
    source_inputs_sha = bindings["source_inputs_manifest_sha256"]
    proto_sha = digest_file(repo / "native/proto/gluerift_native.proto")
    go_sha = digest_file(args.go)
    cargo_sha = digest_file(args.cargo)
    rustc_sha = digest_file(args.rustc)
    linker_sha = digest_file(args.linker)
    cc_sha = digest_file(args.cc)
    platform_descriptor = {
        "go_sha256": go_sha,
        "cc_sha256": cc_sha,
        "kernel": platform.release(),
        "linker_sha256": linker_sha,
        "machine": platform.machine(),
        "platform": platform.platform(),
        "rustc_sha256": rustc_sha,
        "system": platform.system(),
        "canonical_tool_paths": {
            "/opt/gluerift/os/usr/bin/cc": cc_sha,
            "/opt/gluerift/os/usr/bin/ld": linker_sha,
            "/opt/gluerift/toolchains/go/bin/go": go_sha,
            "/opt/gluerift/toolchains/rust/bin/cargo": cargo_sha,
            "/opt/gluerift/toolchains/rust/bin/rustc": rustc_sha,
        },
    }
    with open(repo / "native/host-toolchain.lock.json", "rb") as handle:
        image_lock = json.load(handle)
    descriptor_sha = digest_bytes(canonical(platform_descriptor))
    if (
        image_lock.get("schema") != f"gluerift.host-toolchain-lock/{SCHEMA_VERSION}"
        or image_lock.get("descriptor") != platform_descriptor
        or image_lock.get("descriptor_sha256") != descriptor_sha
        or image_lock.get("host_toolchain_descriptor_sha256") != descriptor_sha
    ):
        raise ValueError("actual OS/toolchain descriptor differs from native/host-toolchain.lock.json")
    descriptor_sha256 = image_lock["host_toolchain_descriptor_sha256"]
    linker_version = command(args.linker, "-v")
    dependency_cache_provisioning = {
        "cargo_lock_sha256": digest_file(repo / "native/rust/Cargo.lock"),
        "cargo_seed_logical_path": ".tools/cargo-home",
        "go_mod_sha256": digest_file(repo / "native/go/go.mod"),
        "go_seed_logical_path": ".tools/gomodcache",
        "go_sum_sha256": digest_file(repo / "native/go/go.sum"),
        "network_mode": "disabled",
        "schema": f"gluerift.dependency-cache-provisioning/{SCHEMA_VERSION}",
        "seed_access": "read-only-copy-to-external-output-cache",
        "toolchain_lock_sha256": digest_file(repo / "native/toolchain.lock.json"),
    }
    dependency_cache_provisioning_sha256 = digest_bytes(canonical(dependency_cache_provisioning))

    role_specs = [
        {
            "role": "go-source",
            "binary": pathlib.Path(args.go_source),
            "logical_binary": "native/bin/gluerift-native-source",
            "compiler": args.go,
            "compiler_image_path": "/opt/gluerift/toolchains/go/bin/go",
            "compiler_sha": go_sha,
            "compiler_version": command(args.go, "version"),
            "compiler_flags": ["build", "-trimpath", "-buildvcs=false", "-mod=readonly", "-o", "native/bin/gluerift-native-source", "./cmd/source"],
            "build_argv": ["build", "-trimpath", "-buildvcs=false", "-mod=readonly", "-o", "native/bin/gluerift-native-source", "./cmd/source"],
            "declared_outputs": ["native/bin/gluerift-native-source"],
            "target": "darwin/arm64",
            "build_environment": {
                **RUNTIME_BASE_ENV,
                "CGO_ENABLED": "0",
                "GOFLAGS": "-mod=readonly",
                "GOCACHE": "/opt/gluerift/output/cache/go-build",
                "GOMODCACHE": "/opt/gluerift/output/cache/gomodcache",
                "GOPATH": "/opt/gluerift/output/cache/gopath",
                "HOME": "/opt/gluerift/output/cache/home",
                "TMPDIR": "/opt/gluerift/output/cache/tmp",
            },
            "roots": ["native/go", "native/proto"],
            "lockfiles": ["native/go/go.mod", "native/go/go.sum"],
        },
        {
            "role": "rust-target",
            "binary": pathlib.Path(args.rust_target),
            "logical_binary": "native/bin/gluerift-native-target",
            "compiler": args.rustc,
            "compiler_image_path": "/opt/gluerift/toolchains/rust/bin/rustc",
            "compiler_sha": rustc_sha,
            "compiler_version": command(args.rustc, "--version", "--verbose"),
            "compiler_flags": [
                "--remap-path-prefix=/workspace=/src",
                "--remap-path-prefix=/opt/gluerift/output=/build",
            ],
            "build_argv": ["build", "--release", "--locked", "--offline", "--manifest-path", "Cargo.toml", "--bin", "gluerift-native-target", "--bin", "gluerift-native-harness"],
            "declared_outputs": ["native/bin/gluerift-native-target", "native/bin/gluerift-native-harness"],
            "target": command(args.rustc, "-vV").split("host: ", 1)[1].splitlines()[0],
            "build_environment": {
                **RUNTIME_BASE_ENV,
                "CARGO_HOME": "/opt/gluerift/output/cache/cargo-home",
                "CARGO_INCREMENTAL": "0",
                "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": "/opt/gluerift/os/usr/bin/cc",
                "CARGO_TARGET_DIR": "/opt/gluerift/output/cargo-target",
                "CC": "/opt/gluerift/os/usr/bin/cc",
                "HOME": "/opt/gluerift/output/cache/home",
                "PATH": "/opt/gluerift/os/usr/bin:/opt/gluerift/os/bin",
                "RUSTC": "/opt/gluerift/toolchains/rust/bin/rustc",
                "RUSTFLAGS": "--remap-path-prefix=/workspace=/src --remap-path-prefix=/opt/gluerift/output=/build",
                "TMPDIR": "/opt/gluerift/output/cache/tmp",
            },
            "roots": ["native/rust", "native/proto"],
            "lockfiles": ["native/rust/Cargo.lock"],
        },
        {
            "role": "native-harness",
            "binary": pathlib.Path(args.native_harness),
            "logical_binary": "native/bin/gluerift-native-harness",
            "compiler": args.rustc,
            "compiler_image_path": "/opt/gluerift/toolchains/rust/bin/rustc",
            "compiler_sha": rustc_sha,
            "compiler_version": command(args.rustc, "--version", "--verbose"),
            "compiler_flags": [
                "--remap-path-prefix=/workspace=/src",
                "--remap-path-prefix=/opt/gluerift/output=/build",
            ],
            "build_argv": ["build", "--release", "--locked", "--offline", "--manifest-path", "Cargo.toml", "--bin", "gluerift-native-target", "--bin", "gluerift-native-harness"],
            "declared_outputs": ["native/bin/gluerift-native-target", "native/bin/gluerift-native-harness"],
            "target": command(args.rustc, "-vV").split("host: ", 1)[1].splitlines()[0],
            "build_environment": {
                **RUNTIME_BASE_ENV,
                "CARGO_HOME": "/opt/gluerift/output/cache/cargo-home",
                "CARGO_INCREMENTAL": "0",
                "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": "/opt/gluerift/os/usr/bin/cc",
                "CARGO_TARGET_DIR": "/opt/gluerift/output/cargo-target",
                "CC": "/opt/gluerift/os/usr/bin/cc",
                "HOME": "/opt/gluerift/output/cache/home",
                "PATH": "/opt/gluerift/os/usr/bin:/opt/gluerift/os/bin",
                "RUSTC": "/opt/gluerift/toolchains/rust/bin/rustc",
                "RUSTFLAGS": "--remap-path-prefix=/workspace=/src --remap-path-prefix=/opt/gluerift/output=/build",
                "TMPDIR": "/opt/gluerift/output/cache/tmp",
            },
            "roots": ["native/rust", "native/proto"],
            "lockfiles": ["native/rust/Cargo.lock"],
        },
    ]
    roles = []
    for spec in role_specs:
        build_env_sha = digest_bytes(canonical({
            "build_environment": spec["build_environment"],
            "build_environment_mode": "empty-plus-whitelist",
        }))
        dependency_manifest = dependencies(spec["binary"], descriptor_sha256)
        dependency_sha = digest_bytes(canonical(dependency_manifest))
        files = source_hashes(repo, spec["roots"])
        lockfiles = {path: digest_file(repo / path) for path in spec["lockfiles"]}
        output_sha = digest_file(spec["binary"])
        build_tool = args.go if spec["role"] == "go-source" else args.cargo
        build_tool_image_path = "/opt/gluerift/toolchains/go/bin/go" if spec["role"] == "go-source" else "/opt/gluerift/toolchains/rust/bin/cargo"
        build_tool_sha = go_sha if spec["role"] == "go-source" else cargo_sha
        manifest = {
            "build_environment": spec["build_environment"],
            "build_environment_mode": "empty-plus-whitelist",
            "build_environment_sha256": build_env_sha,
            "build_steps": [{
                "argv": spec["build_argv"],
                "declared_input_hashes": files,
                "declared_output_logical_paths": spec["declared_outputs"],
                "environment_sha256": build_env_sha,
                "step_id": "compile-and-link",
                "tool_absolute_path": build_tool_image_path,
                "tool_executable_sha256": build_tool_sha,
                "working_directory": "native/go" if spec["role"] == "go-source" else "native/rust",
            }],
            "compiler_absolute_path": spec["compiler_image_path"],
            "compiler_executable_sha256": spec["compiler_sha"],
            "compiler_flags": spec["compiler_flags"],
            "compiler_version": spec["compiler_version"],
            "host_toolchain_descriptor_sha256": descriptor_sha256,
            "dynamic_dependency_manifest_sha256": dependency_sha,
            "dependency_cache_provisioning_sha256": dependency_cache_provisioning_sha256,
            "linker_absolute_path": "/opt/gluerift/os/usr/bin/ld",
            "linker_executable_sha256": linker_sha,
            "linker_flags": [],
            "linker_version": linker_version,
            "lockfile_hashes": lockfiles,
            "network_mode": "disabled",
            "output_executable_sha256": output_sha,
            "output_logical_path": spec["logical_binary"],
            "proto_schema_sha256": proto_sha,
            "schema": f"gluerift.build-manifest/{SCHEMA_VERSION}",
            "source_tree_read_only": True,
            "source_tree_read_only_enforcement": "sandbox-exec-tested-output-only-write-whitelist",
            "source_file_hashes": files,
            "source_inputs_manifest_sha256": source_inputs_sha,
            "source_tree_sha256": source_tree_sha,
            "target_triple": spec["target"],
        }
        manifest_sha = digest_bytes(canonical(manifest))
        roles.append({
            "build_manifest": manifest,
            "build_manifest_sha256": manifest_sha,
            "dynamic_dependency_manifest": dependency_manifest,
            "dynamic_dependency_manifest_sha256": dependency_sha,
            "executable_logical_path": spec["logical_binary"],
            "executable_sha256": output_sha,
            "role": spec["role"],
        })
    evidence = {
        "host_toolchain_descriptor_sha256": descriptor_sha256,
        "dependency_cache_provisioning": dependency_cache_provisioning,
        "dependency_cache_provisioning_sha256": dependency_cache_provisioning_sha256,
        "roles": sorted(roles, key=lambda value: value["role"]),
        "schema": f"gluerift.native-build-evidence/{SCHEMA_VERSION}",
    }
    pathlib.Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "wb") as handle:
        handle.write(canonical(evidence))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"build-evidence error: {error}", file=sys.stderr)
        raise
