use anyhow::{Context, Result, bail};
use gluerift_native::canonical;
use gluerift_native::conformance::{NativeExecutables, ensure_success, run};
use gluerift_native::evidence::{
    BuildEvidence, NativeOutputIndex, ReferenceBindings, emit, validate_required_laws,
};
use gluerift_native::reference::NativeReferenceBundle;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Args {
    bindings: PathBuf,
    build_evidence: PathBuf,
    go_source: PathBuf,
    logical_out_prefix: String,
    network_isolation: String,
    out_dir: PathBuf,
    repo: PathBuf,
    rust_target: PathBuf,
}

fn parse_args() -> Result<Args> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    let command = args.next().context("expected reproduce command")?;
    if command != "reproduce" {
        bail!("only the reproduce command is supported")
    }
    while let Some(flag) = args.next() {
        if !matches!(
            flag.as_str(),
            "--bindings"
                | "--build-evidence"
                | "--go-source"
                | "--logical-out-prefix"
                | "--network-isolation"
                | "--out-dir"
                | "--repo"
                | "--rust-target"
        ) {
            bail!("unknown argument {flag}")
        }
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        if values.insert(flag.clone(), value).is_some() {
            bail!("duplicate argument {flag}")
        }
    }
    let take = |map: &mut BTreeMap<String, String>, key: &str| -> Result<String> {
        map.remove(key).with_context(|| format!("missing {key}"))
    };
    Ok(Args {
        bindings: take(&mut values, "--bindings")?.into(),
        build_evidence: take(&mut values, "--build-evidence")?.into(),
        go_source: take(&mut values, "--go-source")?.into(),
        logical_out_prefix: take(&mut values, "--logical-out-prefix")?,
        network_isolation: take(&mut values, "--network-isolation")?,
        out_dir: take(&mut values, "--out-dir")?.into(),
        repo: take(&mut values, "--repo")?.into(),
        rust_target: take(&mut values, "--rust-target")?.into(),
    })
}

fn write_build_evidence(out: &Path, build: &BuildEvidence, logical_prefix: &str) -> Result<String> {
    let build_dir = out.join("build");
    fs::create_dir_all(&build_dir)?;
    let mut roles: Vec<_> = build.roles.iter().collect();
    roles.sort_by_key(|role| role.role.as_str());
    let mut index = Vec::new();
    let provisioning_name = "dependency-cache-provisioning.json";
    let provisioning_hash = canonical::write_file(
        &build_dir.join(provisioning_name),
        &build.dependency_cache_provisioning,
    )?;
    if provisioning_hash != build.dependency_cache_provisioning_sha256 {
        bail!("dependency-cache provisioning object changed after validation")
    }
    for role in roles {
        let manifest_name = format!("{}.build-manifest.json", role.role);
        let dependency_name = format!("{}.dynamic-dependencies.json", role.role);
        let manifest_hash =
            canonical::write_file(&build_dir.join(&manifest_name), &role.build_manifest)?;
        let dependency_hash = canonical::write_file(
            &build_dir.join(&dependency_name),
            &role.dynamic_dependency_manifest,
        )?;
        if manifest_hash != role.build_manifest_sha256
            || dependency_hash != role.dynamic_dependency_manifest_sha256
        {
            bail!("build-evidence object changed after validation")
        }
        index.push(json!({
            "build_manifest_logical_path": format!("{logical_prefix}/build/{manifest_name}"),
            "build_manifest_sha256": manifest_hash,
            "dynamic_dependency_manifest_logical_path": format!("{logical_prefix}/build/{dependency_name}"),
            "dynamic_dependency_manifest_sha256": dependency_hash,
            "role": role.role
        }));
    }
    canonical::write_file(
        &build_dir.join("index.json"),
        &json!({
            "dependency_cache_provisioning_logical_path": format!("{logical_prefix}/build/{provisioning_name}"),
            "dependency_cache_provisioning_sha256": provisioning_hash,
            "entries": index,
            "schema": "gluerift.native-build-index/v0.3.1a"
        }),
    )
}

fn execute(args: Args) -> Result<()> {
    if !args.repo.is_dir() || !args.go_source.is_file() || !args.rust_target.is_file() {
        bail!("repository or native executable path does not exist")
    }
    if !matches!(
        args.network_isolation.as_str(),
        "sandbox-exec" | "outer-sandbox-exec" | "outer"
    ) {
        bail!("--network-isolation must be sandbox-exec, outer-sandbox-exec, or outer")
    }
    let bindings = ReferenceBindings::read(&args.bindings)?;
    let build = BuildEvidence::read(&args.build_evidence)?;
    build.validate_source_bindings(&bindings)?;
    let harness = std::env::current_exe().context("resolve native harness executable")?;
    build.validate_binaries(&args.go_source, &args.rust_target, &harness)?;
    if args.network_isolation == "sandbox-exec" {
        gluerift_native::process::verify_source_read_only(&args.repo)?;
    }
    let executables = NativeExecutables {
        go_source: args.go_source,
        rust_target: args.rust_target,
        repo: args.repo,
        use_sandbox_exec: args.network_isolation == "sandbox-exec",
        outer_isolation: args.network_isolation.clone(),
    };
    let mut fixture_entries = Vec::new();
    for fixture in ["E01", "E02"] {
        let binding = bindings.reference(fixture)?;
        let bundle = NativeReferenceBundle::read(&executables.repo, &bindings, binding)?;
        let semantic = run(
            fixture,
            &executables,
            &bundle,
            &binding.reference_bundle_sha256,
        )?;
        ensure_success(&semantic)?;
        validate_required_laws(&semantic)?;
        fixture_entries.push(emit(
            &semantic,
            &bindings,
            &build,
            &executables.repo,
            &args.out_dir,
            &args.logical_out_prefix,
        )?);
    }
    let build_index_sha256 = write_build_evidence(&args.out_dir, &build, &args.logical_out_prefix)?;
    let index = NativeOutputIndex {
        fixtures: fixture_entries,
        schema: "gluerift.native-output-index/v0.3.1a".to_owned(),
    };
    let index_sha256 = canonical::write_file(&args.out_dir.join("index.json"), &index)?;
    let summary = json!({
        "build_index_sha256": build_index_sha256,
        "index_sha256": index_sha256,
        "status": "proved-exhaustive"
    });
    println!("{}", String::from_utf8(canonical::to_vec(&summary)?)?);
    Ok(())
}

fn main() {
    if let Err(error) = parse_args().and_then(execute) {
        eprintln!("native harness error: {error:#}");
        std::process::exit(4);
    }
}
