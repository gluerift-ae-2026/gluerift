#!/usr/bin/env python3
"""Hermetically reproduce the GlueRift v0.3.1a Minimal Core release.

The checked-in source tree is never used as an output directory.  A verified
snapshot is copied to an external staging root, all build/cache roots are kept
next to that snapshot, and every semantic command is run with networking and
writes outside its declared output root denied by macOS sandbox-exec.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
from pathlib import Path, PurePosixPath
from typing import Iterable, Mapping, Sequence

from jcs import canonical_sha256, load_json
from source_manifest import build_manifest, verify_entries


class ReproductionError(RuntimeError):
    pass


CONTRACT = "ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md"
SOURCE_DATE_EPOCH = "1767225600"
BASE_ENV = {
    "LANG": "C.UTF-8",
    "LC_ALL": "C.UTF-8",
    "SOURCE_DATE_EPOCH": SOURCE_DATE_EPOCH,
    "TZ": "UTC",
}

PAPER_TEX_NAMES = (
    "core-results-table.tex",
    "regressions-table.tex",
    "native-table.tex",
    "bl4-delta-table.tex",
)

CHECKED_OWNER_PATHS = (
    "artifact/reproduction-manifest.json",
    "artifact/results/results.json",
    "artifact/claims.json",
    "artifact/tables/core-results.tsv",
    *(f"paper/generated/{name}" for name in PAPER_TEX_NAMES),
)


def log(message: str) -> None:
    print(f"[GlueRift] {message}", flush=True)


def require_bootstrapped_tools(root: Path) -> None:
    required = [
        root / "proof/.toolchain/bin/lean",
        root / ".tools/go1.26.1/bin/go",
        root / ".tools/protoc-35.0/bin/protoc",
        root / ".tools/bin/protoc-gen-go",
        root / ".tools/bin/cargo",
        root / ".tools/bin/rustc",
        root / ".tools/bin/rustdoc",
        root / ".tools/bin/cc",
        root / ".tools/bin/ld",
    ]
    missing = [path.relative_to(root).as_posix() for path in required if not path.is_file()]
    if missing:
        raise ReproductionError(
            "pinned tools are not provisioned; run ./artifact/bootstrap-tools first; "
            f"missing: {missing}"
        )


def ensure_external_empty(root: Path, output: Path) -> Path:
    output = output.expanduser().resolve()
    try:
        output.relative_to(root.resolve())
    except ValueError:
        pass
    else:
        raise ReproductionError("--out-dir must be outside the checked-in source tree")
    if output.exists():
        if not output.is_dir():
            raise ReproductionError(f"output exists and is not a directory: {output}")
        if any(output.iterdir()):
            raise ReproductionError(f"output directory must be empty: {output}")
    else:
        output.mkdir(parents=True)
    return output


def run(
    argv: Sequence[str | Path],
    *,
    cwd: Path,
    env: Mapping[str, str] | None = None,
    capture: Path | None = None,
) -> None:
    command = [str(item) for item in argv]
    display = " ".join(command)
    log(display)
    full_env = dict(BASE_ENV)
    if env:
        full_env.update({key: str(value) for key, value in env.items()})
    if capture is None:
        completed = subprocess.run(command, cwd=cwd, env=full_env, check=False)
    else:
        capture.parent.mkdir(parents=True, exist_ok=True)
        with capture.open("wb") as stream:
            completed = subprocess.run(
                command,
                cwd=cwd,
                env=full_env,
                stdout=stream,
                stderr=subprocess.STDOUT,
                check=False,
            )
    if completed.returncode != 0:
        suffix = f"; log: {capture}" if capture else ""
        raise ReproductionError(
            f"command exited {completed.returncode}: {display}{suffix}"
        )


def sandbox_profile(write_roots: Iterable[Path]) -> str:
    roots = sorted({str(path.resolve()) for path in write_roots})
    if not roots:
        raise ReproductionError("sandbox requires at least one external write root")
    permitted = " ".join(f'(subpath "{path}")' for path in roots)
    return (
        "(version 1)"
        "(allow default)"
        "(deny network*)"
        "(deny file-write* "
        " (require-all"
        f"  (require-not (require-any {permitted}))"
        "  (require-not (literal \"/dev/null\"))"
        "  (require-not (literal \"/dev/urandom\"))"
        " ))"
    )


def sandboxed(
    argv: Sequence[str | Path],
    *,
    cwd: Path,
    write_roots: Iterable[Path],
    env: Mapping[str, str] | None = None,
    capture: Path | None = None,
) -> None:
    profile = sandbox_profile(write_roots)
    child_environment = dict(BASE_ENV)
    if env:
        child_environment.update({key: str(value) for key, value in env.items()})
    assignments = [
        f"{key}={value}" for key, value in sorted(child_environment.items())
    ]
    run(
        [
            "/usr/bin/sandbox-exec",
            "-p",
            profile,
            "/usr/bin/env",
            "-i",
            *assignments,
            *argv,
        ],
        cwd=cwd,
        capture=capture,
    )


def copy_source_snapshot(source: Path, release: Path, manifest: dict) -> None:
    for entry in manifest["entries"]:
        logical = PurePosixPath(entry["logical_path"])
        source_path = source.joinpath(*logical.parts)
        target_path = release.joinpath(*logical.parts)
        target_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_path, target_path)
        mode = target_path.stat().st_mode
        if entry["executable_bit"]:
            target_path.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        else:
            target_path.chmod(mode & ~(stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH))
    verify_entries(release, manifest)


def compare_trees(left: Path, right: Path, label: str) -> None:
    def files(root: Path) -> list[str]:
        return sorted(
            PurePosixPath(*path.relative_to(root).parts).as_posix()
            for path in root.rglob("*")
            if path.is_file()
        )

    left_files = files(left)
    right_files = files(right)
    if left_files != right_files:
        raise ReproductionError(f"{label}: deterministic path sets differ")
    for logical in left_files:
        if (left / logical).read_bytes() != (right / logical).read_bytes():
            raise ReproductionError(f"{label}: bytes differ at {logical}")
    log(f"{label}: byte-identical ({len(left_files)} files)")


def copy_tree_no_overwrite(source: Path, target: Path) -> None:
    if target.exists():
        raise ReproductionError(f"refusing to overwrite generated staging path: {target}")
    shutil.copytree(source, target)


def python(root: Path, script: str, *args: str | Path) -> list[str | Path]:
    return [sys.executable, root / "artifact/lib" / script, *args]


def proof_phase(source: Path, release: Path, build: Path) -> Path:
    log("building and auditing Lean L1-L9")
    proof_build = build / "proof"
    shutil.copytree(release / "proof", proof_build)
    toolchain = source / "proof/.toolchain"
    if not (toolchain / "bin/lean").is_file():
        raise ReproductionError("pinned Lean toolchain is not provisioned; run proof/bootstrap.sh")
    (proof_build / ".toolchain").symlink_to(toolchain, target_is_directory=True)
    audit_log = build / "logs/lean-audit.log"
    sandboxed(
        ["/bin/bash", proof_build / "audit.sh"],
        cwd=proof_build,
        write_roots=[proof_build, build / "logs"],
        env={"PATH": f"{toolchain / 'bin'}:/opt/homebrew/bin:/usr/bin:/bin"},
        capture=audit_log,
    )
    proof_evidence = release / "artifact/evidence/proof"
    proof_evidence.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(audit_log, proof_evidence / "axiom-audit.log")
    run(
        python(
            release,
            "proof_audit_report.py",
            "--proof-root",
            release / "proof",
            "--audit-log",
            audit_log,
            "--lean-executable",
            toolchain / "bin/lean",
            "--run-configuration",
            release / "spec/run-config/core-v0.3.1a.json",
            "--out",
            proof_evidence / "proof-audit.json",
        ),
        cwd=release,
    )
    return proof_evidence / "proof-audit.json"


def checker_phase(source: Path, release: Path, build: Path) -> Path:
    log("building the pinned Rust checker and running unit/conformance tests")
    cargo = source / ".tools/bin/cargo"
    rustc = source / ".tools/bin/rustc"
    rustdoc = source / ".tools/bin/rustdoc"
    cc = source / ".tools/bin/cc"
    if not all(path.is_file() for path in (cargo, rustc, rustdoc, cc)):
        raise ReproductionError("pinned Rust 1.95.0 toolchain links are not provisioned")
    cargo_home = build / "cargo-home"
    shutil.copytree(source / ".tools/cargo-home", cargo_home)
    target = build / "checker-target"
    home = build / "checker-home"
    temp = build / "checker-tmp"
    for path in (target, home, temp):
        path.mkdir(parents=True, exist_ok=True)
    env = {
        "CARGO_HOME": str(cargo_home),
        "CARGO_INCREMENTAL": "0",
        "CARGO_TARGET_DIR": str(target),
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER": str(cc),
        "CC": str(cc),
        "HOME": str(home),
        "PATH": f"{source / '.tools/bin'}:/usr/bin:/bin",
        "RUSTC": str(rustc),
        "RUSTDOC": str(rustdoc),
        "RUSTFLAGS": f"--remap-path-prefix={release}=/src --remap-path-prefix={build}=/build",
        "RUSTUP_TOOLCHAIN": "1.95.0-aarch64-apple-darwin",
        "TMPDIR": str(temp),
    }
    common = [
        cargo,
        "test",
        "--release",
        "--locked",
        "--offline",
        "--manifest-path",
        release / "checker/Cargo.toml",
    ]
    sandboxed(common, cwd=release, write_roots=[build], env=env)
    sandboxed(
        [
            cargo,
            "build",
            "--release",
            "--locked",
            "--offline",
            "--manifest-path",
            release / "checker/Cargo.toml",
            "--bin",
            "gluerift",
        ],
        cwd=release,
        write_roots=[build],
        env=env,
    )
    executable = target / "release/gluerift"
    if not executable.is_file():
        raise ReproductionError("Rust checker build did not produce gluerift")
    return executable


def semantic_phase(release: Path, output: Path, executable: Path) -> tuple[Path, Path]:
    output.mkdir(parents=True, exist_ok=True)
    semantic = output / "semantic"
    baseline_directory = output / "baselines"
    baseline_index = output / "baseline-results.json"
    if semantic.exists() or baseline_directory.exists() or baseline_index.exists():
        raise ReproductionError(f"semantic output paths are not empty under {output}")
    semantic.mkdir()
    sandboxed(
        [executable, "run-fixtures", "--registry", release / "fixtures/registry.json", "--out-dir", semantic],
        cwd=release,
        write_roots=[semantic],
    )
    sandboxed(
        [executable, "run-baselines", "--registry", release / "fixtures/registry.json", "--baselines", "BL2,BL4", "--out-dir", output],
        cwd=release,
        write_roots=[baseline_directory, baseline_index],
    )
    return semantic, baseline_index


def native_phase(source: Path, release: Path, build: Path, evidence: Path, bindings: Path) -> Path:
    tools_link = release / ".tools"
    if not tools_link.exists():
        tools_link.symlink_to(source / ".tools", target_is_directory=True)
    native_build = build / "native"
    run(
        [
            release / "native/scripts/reproduce",
            "--bindings",
            bindings,
            "--out-dir",
            evidence,
            "--build-dir",
            native_build,
            "--logical-out-prefix",
            "artifact/evidence/native",
            "--network-isolation",
            "sandbox-exec",
        ],
        cwd=release,
        env={"PATH": "/usr/bin:/bin"},
    )
    return evidence / "index.json"


def schema_and_canonical_audit(release: Path) -> None:
    run(
        python(
            release,
            "schema_check.py",
            "--schema-dir",
            release / "spec/schema",
            "--instances",
            release / "spec/run-config",
            "--instances",
            release / "spec/transformation-families",
            "--instances",
            release / "fixtures",
            "--instances",
            release / "baselines",
            "--instances",
            release / "artifact/claim-spec.json",
            "--instances",
            release / "native/host-toolchain.lock.json",
            "--instances",
            release / "artifact/evidence",
            "--instances",
            release / "artifact/claims.json",
            "--instances",
            release / "artifact/results/results.json",
            "--instances",
            release / "artifact/source-inputs.manifest.json",
            "--instances",
            release / "artifact/reproduction-manifest.json",
        ),
        cwd=release,
    )
    canonical_paths = [
        release / "spec/schema",
        release / "spec/run-config",
        release / "spec/transformation-families",
        release / "fixtures",
        release / "baselines",
        release / "artifact/claim-spec.json",
        release / "native/host-toolchain.lock.json",
        release / "native/toolchain.lock.json",
        release / "proof/lake-manifest.json",
        release / "proof/toolchain-lock.json",
        release / "artifact/source-inputs.manifest.json",
        release / "artifact/evidence",
        release / "artifact/claims.json",
        release / "artifact/results/results.json",
        release / "artifact/reproduction-manifest.json",
    ]
    run(
        python(release, "canonical_audit.py", "--require-canonical-bytes", *canonical_paths),
        cwd=release,
    )


def primary_preflight(root: Path) -> None:
    log("preflight: validating schemas, fixtures, canonical bytes, and host/toolchain lock")
    run([sys.executable, root / "fixtures/validate.py"], cwd=root)
    run(
        python(
            root,
            "image_lock.py",
            "--root",
            root,
            "--lock",
            root / "native/host-toolchain.lock.json",
        ),
        cwd=root,
    )
    run(
        python(
            root,
            "schema_check.py",
            "--schema-dir",
            root / "spec/schema",
            "--instances",
            root / "spec/run-config",
            "--instances",
            root / "spec/transformation-families",
            "--instances",
            root / "fixtures",
            "--instances",
            root / "baselines",
            "--instances",
            root / "artifact/claim-spec.json",
            "--instances",
            root / "native/host-toolchain.lock.json",
        ),
        cwd=root,
    )
    run(
        python(
            root,
            "canonical_audit.py",
            "--require-canonical-bytes",
            root / "spec/schema",
            root / "spec/run-config",
            root / "spec/transformation-families",
            root / "fixtures",
            root / "baselines",
            root / "artifact/claim-spec.json",
            root / "native/host-toolchain.lock.json",
            root / "native/toolchain.lock.json",
            root / "proof/lake-manifest.json",
            root / "proof/toolchain-lock.json",
        ),
        cwd=root,
    )


def checked_release_preflight(root: Path) -> None:
    required = [
        root / "artifact/source-inputs.manifest.json",
        *(root / logical for logical in CHECKED_OWNER_PATHS),
    ]
    missing = [str(path) for path in required if not path.is_file()]
    if missing:
        raise ReproductionError(f"checked release preflight is incomplete: {missing}")
    verify_entries(root, load_json(root / "artifact/source-inputs.manifest.json"))
    schema_and_canonical_audit(root)
    run(
        python(
            root,
            "evidence_graph.py",
            "--root",
            root,
            "--manifest",
            root / "artifact/reproduction-manifest.json",
        ),
        cwd=root,
    )
    run(
        python(
            root,
            "claim_check.py",
            "--root",
            root,
            "--claims",
            root / "artifact/claims.json",
            "--results",
            root / "artifact/results/results.json",
        ),
        cwd=root,
    )
    run(
        python(
            root,
            "generate_table.py",
            "--results",
            root / "artifact/results/results.json",
            "--compare",
            root / "artifact/tables/core-results.tsv",
        ),
        cwd=root,
    )
    run(
        python(
            root,
            "generate_paper_tex.py",
            "--results",
            root / "artifact/results/results.json",
            "--compare-dir",
            root / "paper/generated",
        ),
        cwd=root,
    )


def checked_release_mode(root: Path) -> bool:
    present = [logical for logical in CHECKED_OWNER_PATHS if (root / logical).is_file()]
    if not present:
        return False
    if len(present) != len(CHECKED_OWNER_PATHS):
        missing = sorted(set(CHECKED_OWNER_PATHS) - set(present))
        raise ReproductionError(
            "release owner files are partial; refusing ambiguous generation/verification "
            f"mode; missing: {missing}"
        )
    return True


def assemble_release(
    release: Path,
    semantic: Path,
    baseline_results: Path,
    native_index: Path,
    proof_audit: Path,
    host_toolchain_descriptor_sha256: str,
) -> None:
    evidence = release / "artifact/evidence"
    results = release / "artifact/results/results.json"
    results.parent.mkdir(parents=True, exist_ok=True)
    run(
        python(
            release,
            "assemble_results.py",
            "--root",
            release,
            "--fixtures",
            semantic / "fixture-results.json",
            "--baselines",
            baseline_results,
            "--native-index",
            native_index,
            "--proof-audit",
            proof_audit,
            "--evidence",
            semantic,
            "--evidence",
            evidence / "baselines",
            "--evidence",
            evidence / "native",
            "--evidence",
            evidence / "proof",
            "--out",
            results,
        ),
        cwd=release,
    )
    claims = release / "artifact/claims.json"
    run(
        python(
            release,
            "generate_claims.py",
            "--spec",
            release / "artifact/claim-spec.json",
            "--results",
            results,
            "--out",
            claims,
        ),
        cwd=release,
    )
    run(
        python(release, "claim_check.py", "--root", release, "--claims", claims, "--results", results),
        cwd=release,
    )
    table = release / "artifact/tables/core-results.tsv"
    table.parent.mkdir(parents=True, exist_ok=True)
    run(python(release, "generate_table.py", "--results", results, "--out", table), cwd=release)
    paper_table_dir = release / "paper/generated"
    run(
        python(
            release,
            "generate_paper_tex.py",
            "--results",
            results,
            "--out-dir",
            paper_table_dir,
        ),
        cwd=release,
    )
    paper_tables = [paper_table_dir / name for name in PAPER_TEX_NAMES]
    run(
        python(
            release,
            "build_reproduction_manifest.py",
            "--root",
            release,
            "--source-manifest",
            release / "artifact/source-inputs.manifest.json",
            "--run-configuration",
            release / "spec/run-config/core-v0.3.1a.json",
            "--transformation-family",
            release / "spec/transformation-families/core-structural-v0.3.1a.json",
            "--fixture-registry",
            release / "fixtures/registry.json",
            "--image-lock",
            release / "native/host-toolchain.lock.json",
            "--generated",
            evidence,
            "--claims",
            claims,
            "--results",
            results,
            "--tables",
            table,
            *(item for paper_table in paper_tables for item in ("--tables", paper_table)),
            "--pinned-host-toolchain-descriptor-sha256",
            host_toolchain_descriptor_sha256,
            "--out",
            release / "artifact/reproduction-manifest.json",
        ),
        cwd=release,
    )


def reproduce(args: argparse.Namespace) -> Path:
    source = args.root.resolve()
    if not (source / CONTRACT).is_file():
        raise ReproductionError(f"frozen contract is absent: {source / CONTRACT}")
    require_bootstrapped_tools(source)
    primary_preflight(source)
    verify_checked_release = False if args.stage_only else checked_release_mode(source)
    if verify_checked_release:
        log("mode: checked-release verification and byte comparison")
        checked_release_preflight(source)
    else:
        log("mode: source-only checked-release generation")
    output = ensure_external_empty(source, args.out_dir)
    release = output / "release"
    build = output / "build"
    repeat = output / "repeat"
    release.mkdir()
    build.mkdir()
    repeat.mkdir()

    log("verifying frozen source inputs and creating an external snapshot")
    manifest = build_manifest(source)
    copy_source_snapshot(source, release, manifest)
    sys.path.insert(0, str(release / "artifact/lib"))
    from jcs import write_canonical

    write_canonical(release / "artifact/source-inputs.manifest.json", manifest)
    run(python(release, "check_hygiene.py", "--root", release), cwd=release)
    run([sys.executable, release / "fixtures/validate.py"], cwd=release)
    run(
        python(
            release,
            "image_lock.py",
            "--root",
            source,
            "--lock",
            release / "native/host-toolchain.lock.json",
        ),
        cwd=release,
    )
    host_toolchain_descriptor_sha256 = load_json(release / "native/host-toolchain.lock.json")[
        "host_toolchain_descriptor_sha256"
    ]

    proof_audit = proof_phase(source, release, build)
    checker = checker_phase(source, release, build)

    log("running semantic fixtures and strongest baselines twice")
    canonical_evidence = release / "artifact/evidence"
    semantic, baselines = semantic_phase(release, canonical_evidence, checker)
    repeat_evidence = repeat / "evidence"
    repeat_semantic, repeat_baselines = semantic_phase(release, repeat_evidence, checker)
    compare_trees(semantic, repeat_semantic, "semantic witness replay")
    compare_trees(canonical_evidence / "baselines", repeat_evidence / "baselines", "BL2/BL4 replay")
    if baselines.read_bytes() != repeat_baselines.read_bytes():
        raise ReproductionError("baseline aggregate is not byte-identical")
    run(
        python(
            release,
            "transformation_audit.py",
            "--root",
            release,
            "--semantic-root",
            semantic,
            "--registry",
            release / "fixtures/registry.json",
        ),
        cwd=release,
    )

    # This policy-owned binding is canonical release evidence, not an ambient
    # build input.  Native processes consume the exact bytes that the graph
    # later binds.
    bindings = canonical_evidence / "native-reference-bindings.json"
    run(
        python(
            release,
            "native_bindings.py",
            "--source-manifest",
            release / "artifact/source-inputs.manifest.json",
            "--semantic-root",
            semantic,
            "--out",
            bindings,
        ),
        cwd=release,
    )
    log("building and replaying E01/E02 twice through shared Protobuf")
    native_external = output / "native-evidence"
    native_index_external = native_phase(source, release, build, native_external, bindings)
    native_repeat = output / "native-evidence-repeat"
    native_phase(source, release, repeat / "native-build", native_repeat, bindings)
    compare_trees(native_external, native_repeat, "native E01/E02 replay")
    copy_tree_no_overwrite(native_external, canonical_evidence / "native")
    native_index = canonical_evidence / "native/index.json"
    if canonical_sha256(load_json(native_index)) != canonical_sha256(load_json(native_index_external)):
        raise ReproductionError("copied native output index changed")

    for fixture_id in ("E01", "E02"):
        native_manifest = load_json(
            canonical_evidence / f"native/{fixture_id}/native-manifest.json"
        )
        if native_manifest["host_toolchain_descriptor_sha256"] != host_toolchain_descriptor_sha256:
            raise ReproductionError(
                f"{fixture_id} native manifest differs from pinned host/toolchain lock"
            )
    assemble_release(
        release,
        semantic,
        baselines,
        native_index,
        proof_audit,
        host_toolchain_descriptor_sha256,
    )
    schema_and_canonical_audit(release)
    run(
        python(
            release,
            "evidence_graph.py",
            "--root",
            release,
            "--manifest",
            release / "artifact/reproduction-manifest.json",
        ),
        cwd=release,
    )
    verify_entries(release, load_json(release / "artifact/source-inputs.manifest.json"))
    verify_entries(source, load_json(release / "artifact/source-inputs.manifest.json"))

    if verify_checked_release:
        run(
            python(
                release,
                "compare_release.py",
                "--expected-root",
                source,
                "--actual-root",
                release,
            ),
            cwd=release,
        )
    log(f"Final Core reproduction passed; staging root: {release}")
    return release


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--out-dir", type=Path)
    parser.add_argument(
        "--stage-only",
        action="store_true",
        help="force checked-release generation without comparing checked-in owner files",
    )
    args = parser.parse_args()
    if args.out_dir is None:
        args.out_dir = Path(tempfile.mkdtemp(prefix="gluerift-reproduce."))
    reproduce(args)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, ReproductionError, subprocess.SubprocessError) as error:
        print(f"reproduction error: {error}", file=sys.stderr)
        raise SystemExit(4)
