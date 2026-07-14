#!/usr/bin/env python3
"""Verify the pinned Darwin host/toolchain descriptor before any build."""

from __future__ import annotations

import argparse
import platform
import subprocess
import sys
from pathlib import Path

from jcs import canonical_sha256, load_json, sha256_bytes


class ImageLockError(ValueError):
    pass


HOST_TO_IMAGE = {
    ".tools/bin/cc": "/opt/gluerift/os/usr/bin/cc",
    ".tools/bin/ld": "/opt/gluerift/os/usr/bin/ld",
    ".tools/go1.26.1/bin/go": "/opt/gluerift/toolchains/go/bin/go",
    ".tools/bin/cargo": "/opt/gluerift/toolchains/rust/bin/cargo",
    ".tools/bin/rustc": "/opt/gluerift/toolchains/rust/bin/rustc",
}


def current_descriptor(root: Path) -> dict:
    hashes = {}
    for host_logical, image_path in HOST_TO_IMAGE.items():
        path = root / host_logical
        if not path.is_file():
            raise ImageLockError(f"pinned tool is absent: {host_logical}")
        hashes[image_path] = sha256_bytes(path.read_bytes())
    return {
        "canonical_tool_paths": dict(sorted(hashes.items())),
        "cc_sha256": hashes["/opt/gluerift/os/usr/bin/cc"],
        "go_sha256": hashes["/opt/gluerift/toolchains/go/bin/go"],
        "kernel": platform.release(),
        "linker_sha256": hashes["/opt/gluerift/os/usr/bin/ld"],
        "machine": platform.machine(),
        "platform": platform.platform(),
        "rustc_sha256": hashes["/opt/gluerift/toolchains/rust/bin/rustc"],
        "system": platform.system(),
    }


def verify(root: Path, lock_path: Path) -> dict:
    lock = load_json(lock_path)
    if lock.get("schema") != "gluerift.host-toolchain-lock/v0.3.1a":
        raise ImageLockError("host/toolchain lock schema mismatch")
    descriptor = current_descriptor(root)
    digest = canonical_sha256(descriptor)
    if descriptor != lock.get("descriptor"):
        raise ImageLockError("actual Darwin host/toolchain descriptor differs from the lock")
    if digest != lock.get("descriptor_sha256"):
        raise ImageLockError("host/toolchain descriptor SHA-256 mismatch")
    if lock.get("host_toolchain_descriptor_sha256") != digest:
        raise ImageLockError("host/toolchain descriptor hash binding mismatch")
    runtime_version = subprocess.run(
        [sys.executable, "--version"],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    ).stdout.strip()
    if runtime_version != lock.get("descriptor_runtime_version"):
        raise ImageLockError("descriptor runtime version differs from the immutable lock")
    return lock


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--lock", type=Path, required=True)
    args = parser.parse_args()
    lock = verify(args.root.resolve(), args.lock.resolve())
    print(lock["host_toolchain_descriptor_sha256"])
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ImageLockError, subprocess.SubprocessError) as error:
        print(f"host-toolchain-lock error: {error}", file=sys.stderr)
        raise SystemExit(4)
