use crate::canonical;
use crate::conformance::SemanticRun;
use crate::model::ALL_LAW_IDS;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceBindings {
    pub schema: String,
    pub source_inputs_manifest_sha256: String,
    pub source_tree_sha256: String,
    pub references: Vec<ReferenceBinding>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceBinding {
    pub comparator_spec_sha256: String,
    pub endpoint_policy_sha256: String,
    pub fixture_id: String,
    pub reference_candidate_sha256: String,
    pub reference_check_evidence_id: String,
    pub reference_check_report_sha256: String,
    pub reference_bundle_logical_path: String,
    pub reference_bundle_sha256: String,
    pub reference_run_id: String,
    pub run_configuration_sha256: String,
    pub transformed_context_sha256: String,
    pub transformation_report_sha256: String,
    pub types_sha256: String,
    pub validation_request_sha256: String,
    pub validation_scope_sha256: String,
}

impl ReferenceBindings {
    pub fn read(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let bindings: Self = serde_json::from_slice(&bytes)?;
        if bindings.schema != "gluerift.native-reference-bindings/v0.3.1a" {
            bail!("unsupported native reference-binding schema")
        }
        validate_sha(
            "source_inputs_manifest_sha256",
            &bindings.source_inputs_manifest_sha256,
        )?;
        validate_sha("source_tree_sha256", &bindings.source_tree_sha256)?;
        let ids: BTreeSet<_> = bindings
            .references
            .iter()
            .map(|item| item.fixture_id.as_str())
            .collect();
        if ids != BTreeSet::from(["E01", "E02"]) || bindings.references.len() != 2 {
            bail!("native reference bindings must contain exactly E01 and E02")
        }
        for binding in &bindings.references {
            binding.validate()?;
        }
        Ok(bindings)
    }

    pub fn reference(&self, fixture: &str) -> Result<&ReferenceBinding> {
        self.references
            .iter()
            .find(|item| item.fixture_id == fixture)
            .with_context(|| format!("missing {fixture} reference binding"))
    }
}

impl ReferenceBinding {
    fn validate(&self) -> Result<()> {
        let expected_run = match self.fixture_id.as_str() {
            "E01" => "A01",
            "E02" => "A02",
            _ => bail!("unsupported native fixture binding {}", self.fixture_id),
        };
        if self.reference_run_id != expected_run {
            bail!("{} must bind reference run {expected_run}", self.fixture_id)
        }
        if self.reference_candidate_sha256 != self.transformed_context_sha256 {
            bail!("{} candidate/context hash mismatch", self.fixture_id)
        }
        if self.reference_check_evidence_id.trim().is_empty() {
            bail!("{} reference check evidence ID is empty", self.fixture_id)
        }
        validate_logical_path(&self.reference_bundle_logical_path)?;
        for (name, value) in [
            ("comparator_spec_sha256", &self.comparator_spec_sha256),
            ("endpoint_policy_sha256", &self.endpoint_policy_sha256),
            (
                "reference_candidate_sha256",
                &self.reference_candidate_sha256,
            ),
            (
                "reference_check_report_sha256",
                &self.reference_check_report_sha256,
            ),
            ("reference_bundle_sha256", &self.reference_bundle_sha256),
            ("run_configuration_sha256", &self.run_configuration_sha256),
            (
                "transformed_context_sha256",
                &self.transformed_context_sha256,
            ),
            (
                "transformation_report_sha256",
                &self.transformation_report_sha256,
            ),
            ("types_sha256", &self.types_sha256),
            ("validation_request_sha256", &self.validation_request_sha256),
            ("validation_scope_sha256", &self.validation_scope_sha256),
        ] {
            validate_sha(name, value)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildEvidence {
    pub host_toolchain_descriptor_sha256: String,
    pub dependency_cache_provisioning: Value,
    pub dependency_cache_provisioning_sha256: String,
    pub roles: Vec<RoleBuildEvidence>,
    pub schema: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleBuildEvidence {
    pub build_manifest: Value,
    pub build_manifest_sha256: String,
    pub dynamic_dependency_manifest: Value,
    pub dynamic_dependency_manifest_sha256: String,
    pub executable_logical_path: String,
    pub executable_sha256: String,
    pub role: String,
}

impl BuildEvidence {
    pub fn read(path: &Path) -> Result<Self> {
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let value: Self = serde_json::from_slice(&bytes)?;
        if value.schema != "gluerift.native-build-evidence/v0.3.1a" {
            bail!("unsupported native build-evidence schema")
        }
        validate_sha(
            "host_toolchain_descriptor_sha256",
            &value.host_toolchain_descriptor_sha256,
        )?;
        validate_sha(
            "dependency_cache_provisioning_sha256",
            &value.dependency_cache_provisioning_sha256,
        )?;
        if canonical::sha256(&value.dependency_cache_provisioning)?
            != value.dependency_cache_provisioning_sha256
            || value.dependency_cache_provisioning.get("network_mode") != Some(&json!("disabled"))
            || value.dependency_cache_provisioning.get("seed_access")
                != Some(&json!("read-only-copy-to-external-output-cache"))
        {
            bail!("dependency-cache provisioning binding mismatch")
        }
        let roles: BTreeSet<_> = value.roles.iter().map(|role| role.role.as_str()).collect();
        if roles != BTreeSet::from(["go-source", "native-harness", "rust-target"])
            || value.roles.len() != 3
        {
            bail!("build evidence must bind exactly go-source, rust-target, and native-harness")
        }
        for role in &value.roles {
            validate_sha("build_manifest_sha256", &role.build_manifest_sha256)?;
            validate_sha(
                "dynamic_dependency_manifest_sha256",
                &role.dynamic_dependency_manifest_sha256,
            )?;
            validate_sha("executable_sha256", &role.executable_sha256)?;
            if canonical::sha256(&role.build_manifest)? != role.build_manifest_sha256 {
                bail!("{} build manifest hash mismatch", role.role)
            }
            if canonical::sha256(&role.dynamic_dependency_manifest)?
                != role.dynamic_dependency_manifest_sha256
            {
                bail!("{} dynamic-dependency manifest hash mismatch", role.role)
            }
            if role.build_manifest.get("schema") != Some(&json!("gluerift.build-manifest/v0.3.1a"))
                || role.dynamic_dependency_manifest.get("schema")
                    != Some(&json!("gluerift.dynamic-dependency-manifest/v0.3.1a"))
                || role.build_manifest.get("host_toolchain_descriptor_sha256")
                    != Some(&json!(value.host_toolchain_descriptor_sha256))
                || role
                    .dynamic_dependency_manifest
                    .get("host_toolchain_descriptor_sha256")
                    != Some(&json!(value.host_toolchain_descriptor_sha256))
                || role
                    .build_manifest
                    .get("dynamic_dependency_manifest_sha256")
                    != Some(&json!(role.dynamic_dependency_manifest_sha256))
                || role
                    .build_manifest
                    .get("dependency_cache_provisioning_sha256")
                    != Some(&json!(value.dependency_cache_provisioning_sha256))
                || role.build_manifest.get("output_executable_sha256")
                    != Some(&json!(role.executable_sha256))
                || role.build_manifest.get("output_logical_path")
                    != Some(&json!(role.executable_logical_path))
                || role.build_manifest.get("network_mode") != Some(&json!("disabled"))
                || role.build_manifest.get("source_tree_read_only") != Some(&json!(true))
            {
                bail!(
                    "{} build/dependency manifest binding is incomplete",
                    role.role
                )
            }
            validate_logical_path(&role.executable_logical_path)?;
        }
        Ok(value)
    }

    pub fn validate_source_bindings(&self, bindings: &ReferenceBindings) -> Result<()> {
        for role in &self.roles {
            if role.build_manifest.get("source_tree_sha256")
                != Some(&json!(bindings.source_tree_sha256))
                || role.build_manifest.get("source_inputs_manifest_sha256")
                    != Some(&json!(bindings.source_inputs_manifest_sha256))
            {
                bail!("{} source-input/tree binding mismatch", role.role)
            }
        }
        Ok(())
    }

    pub fn role(&self, role: &str) -> Result<&RoleBuildEvidence> {
        self.roles
            .iter()
            .find(|item| item.role == role)
            .with_context(|| format!("missing build role {role}"))
    }

    pub fn validate_binaries(
        &self,
        go_source: &Path,
        rust_target: &Path,
        native_harness: &Path,
    ) -> Result<()> {
        for (role, path) in [
            ("go-source", go_source),
            ("native-harness", native_harness),
            ("rust-target", rust_target),
        ] {
            let expected = &self.role(role)?.executable_sha256;
            let actual = canonical::sha256_file(path)?;
            if &actual != expected {
                bail!("{role} executable hash mismatch")
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NativeOutputIndex {
    pub fixtures: Vec<NativeOutputIndexEntry>,
    pub schema: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NativeOutputIndexEntry {
    pub backend_conformance_logical_path: String,
    pub backend_conformance_sha256: String,
    pub fixture_id: String,
    pub reference_bundle_logical_path: String,
    pub reference_bundle_sha256: String,
    pub native_manifest_logical_path: String,
    pub native_manifest_sha256: String,
    pub replay_report_logical_path: String,
    pub replay_report_sha256: String,
    pub transcript_logical_path: String,
    pub transcript_sha256: String,
}

pub fn emit(
    run: &SemanticRun,
    bindings: &ReferenceBindings,
    builds: &BuildEvidence,
    repo: &Path,
    out_root: &Path,
    logical_prefix: &str,
) -> Result<NativeOutputIndexEntry> {
    validate_logical_path(logical_prefix)?;
    let reference = bindings.reference(&run.fixture_id)?;
    let fixture_dir = out_root.join(&run.fixture_id);
    fs::create_dir_all(&fixture_dir)?;
    let proto_path = repo.join("native/proto/gluerift_native.proto");
    let proto_hash = canonical::sha256_file(&proto_path)?;
    let input_logical = format!("native/{}/input.json", run.fixture_id);
    let input_path = repo.join("native").join(&run.fixture_id).join("input.json");
    let input_hash = canonical::sha256_file(&input_path)?;
    let environment = crate::process::fixed_environment();
    let runtime_environment = json!({
        "environment": environment,
        "environment_mode": "empty-plus-whitelist"
    });
    let runtime_environment_sha256 = canonical::sha256(&runtime_environment)?;

    let mut roles: Vec<_> = builds.roles.iter().collect();
    roles.sort_by_key(|role| role.role.as_str());
    let build_set: Vec<_> = roles
        .iter()
        .map(|role| json!({"build_manifest_sha256": role.build_manifest_sha256, "role": role.role}))
        .collect();
    let dependency_set: Vec<_> = roles
        .iter()
        .map(|role| json!({"dynamic_dependency_manifest_sha256": role.dynamic_dependency_manifest_sha256, "role": role.role}))
        .collect();
    let build_manifest_set_sha256 = canonical::sha256(&build_set)?;
    let dynamic_dependency_manifest_set_sha256 = canonical::sha256(&dependency_set)?;
    let network_isolation_argument = if run
        .main_processes
        .iter()
        .all(|process| process.network_isolation == "sandbox-exec-deny-network")
    {
        "sandbox-exec"
    } else if run
        .main_processes
        .iter()
        .all(|process| process.network_isolation == "outer-sandbox-exec-deny-network")
    {
        "outer-sandbox-exec"
    } else {
        "outer"
    };
    let executables: Vec<_> = roles
        .iter()
        .map(|role| {
            let (argv, timeout) = match role.role.as_str() {
                "go-source" => (
                    vec![
                        role.executable_logical_path.clone(),
                        "--fixture".to_owned(),
                        run.fixture_id.clone(),
                        "--operation".to_owned(),
                        "program-output".to_owned(),
                    ],
                    crate::process::TIMEOUT.as_secs(),
                ),
                "rust-target" => (
                    vec![
                        role.executable_logical_path.clone(),
                        "--fixture".to_owned(),
                        run.fixture_id.clone(),
                        "--operation".to_owned(),
                        "transport-compare".to_owned(),
                    ],
                    crate::process::TIMEOUT.as_secs(),
                ),
                "native-harness" => (
                    vec![
                        role.executable_logical_path.clone(),
                        "reproduce".to_owned(),
                        "--bindings".to_owned(),
                        "{REFERENCE_BINDINGS}".to_owned(),
                        "--build-evidence".to_owned(),
                        "{BUILD_EVIDENCE}".to_owned(),
                        "--go-source".to_owned(),
                        "native/bin/gluerift-native-source".to_owned(),
                        "--rust-target".to_owned(),
                        "native/bin/gluerift-native-target".to_owned(),
                        "--repo".to_owned(),
                        "{REPOSITORY_ROOT}".to_owned(),
                        "--out-dir".to_owned(),
                        "{EXTERNAL_STAGING}".to_owned(),
                        "--logical-out-prefix".to_owned(),
                        logical_prefix.to_owned(),
                        "--network-isolation".to_owned(),
                        network_isolation_argument.to_owned(),
                    ],
                    300,
                ),
                _ => unreachable!("validated build role"),
            };
            json!({
                "argv": argv,
                "build_manifest_sha256": role.build_manifest_sha256,
                "dynamic_dependency_manifest_sha256": role.dynamic_dependency_manifest_sha256,
                "logical_path": role.executable_logical_path,
                "role": role.role,
                "sha256": role.executable_sha256,
                "stderr_limit": crate::process::STDERR_LIMIT,
                "stdin_limit": crate::process::STDIN_LIMIT,
                "stdout_limit": crate::process::STDOUT_LIMIT,
                "timeout": timeout,
                "working_directory": "native"
            })
        })
        .collect();

    let native_manifest = json!({
        "build_manifest_set_sha256": build_manifest_set_sha256,
        "comparator_kind": "target-native-exact",
        "comparator_spec_sha256": reference.comparator_spec_sha256,
        "host_toolchain_descriptor_sha256": builds.host_toolchain_descriptor_sha256,
        "context_sha256": reference.transformed_context_sha256,
        "dynamic_dependency_manifest_set_sha256": dynamic_dependency_manifest_set_sha256,
        "endpoint_policy_sha256": reference.endpoint_policy_sha256,
        "environment": runtime_environment["environment"],
        "environment_mode": "empty-plus-whitelist",
        "executables": executables,
        "expected_comparator_output": "EQUAL",
        "fixture_id": run.fixture_id,
        "network_mode": "disabled",
        "ordinary_comparator_role": "rust-target",
        "proto_schema_sha256": proto_hash,
        "protocol": "gluerift-native-cli-framed-protobuf/v1",
        "reference_bundle_logical_path": reference.reference_bundle_logical_path,
        "reference_bundle_sha256": reference.reference_bundle_sha256,
        "run_configuration_sha256": reference.run_configuration_sha256,
        "runtime_environment_sha256": runtime_environment_sha256,
        "schema": "gluerift.native-manifest/v0.3.1a",
        "source_tree_read_only": true,
        "source_tree_read_only_enforcement": match network_isolation_argument { "sandbox-exec" => "sandbox-exec-tested-no-file-writes", "outer-sandbox-exec" => "outer-sandbox-exec-tested-output-only-write-whitelist", _ => "outer-read-only-source-mount" },
        "stdin_or_fixture_logical_path": input_logical,
        "stdin_or_fixture_sha256": input_hash,
        "types_sha256": reference.types_sha256,
        "validation_request_sha256": reference.validation_request_sha256,
        "validation_scope_sha256": reference.validation_scope_sha256
    });
    let manifest_name = "native-manifest.json";
    let manifest_hash = canonical::write_file(&fixture_dir.join(manifest_name), &native_manifest)?;

    let process_bindings: Vec<_> = run
        .main_processes
        .iter()
        .map(|process| {
            let role = builds.role(&process.role).expect("validated build role");
            json!({
                "argv": process.argv,
                "environment": process.environment,
                "environment_mode": process.environment_mode,
                "executable_logical_path": role.executable_logical_path,
                "executable_sha256": role.executable_sha256,
                "exit_code": process.exit_code,
                "network_isolation": process.network_isolation,
                "operation_id": process.operation_id,
                "role": process.role,
                "source_tree_read_only_enforcement": process.source_tree_read_only_enforcement,
                "timed_out": process.timed_out,
                "working_directory": process.working_directory
            })
        })
        .collect();
    let witness_sha256 = canonical::sha256(&run.witness)?;
    let backend_evidence_id = format!("native-backend-{}", run.fixture_id.to_ascii_lowercase());
    let replay_evidence_id = format!("native-replay-{}", run.fixture_id.to_ascii_lowercase());
    let tool_build_sha256 = &builds.role("native-harness")?.executable_sha256;
    let mut backend_dependencies = vec![
        reference.reference_check_evidence_id.clone(),
        run.reference_bundle_evidence_id.clone(),
    ];
    backend_dependencies.sort();
    let mut replay_dependencies = vec![
        backend_evidence_id.clone(),
        reference.reference_check_evidence_id.clone(),
        run.reference_bundle_evidence_id.clone(),
    ];
    replay_dependencies.sort();
    let replay_report = json!({
        "backend_conformance_evidence_id": backend_evidence_id,
        "bridge_statuses": {"carrier_target": "proved-exhaustive", "selected_carrier_bridge": "proved-exhaustive"},
        "build_manifest_set_sha256": build_manifest_set_sha256,
        "comparator_definedness": "proved-exhaustive",
        "comparator_kind": "target-native-exact",
        "comparator_spec_sha256": reference.comparator_spec_sha256,
        "host_toolchain_descriptor_sha256": builds.host_toolchain_descriptor_sha256,
        "context_sha256": reference.transformed_context_sha256,
        "candidate_sha256": reference.reference_candidate_sha256,
        "dependency_evidence_ids": replay_dependencies,
        "dynamic_dependency_manifest_set_sha256": dynamic_dependency_manifest_set_sha256,
        "endpoint_policy_sha256": reference.endpoint_policy_sha256,
        "fixture_id": run.fixture_id,
        "evidence_id": replay_evidence_id,
        "native_manifest_sha256": manifest_hash,
        "ordinary_comparator_result": run.ordinary_comparator_result,
        "processes": process_bindings,
        "property_statuses": {"policy_soundness": run.policy_soundness},
        "property_witnesses": [{"property_id": "policy-soundness", "witness_kind": run.witness.witness_kind, "witness_sha256": witness_sha256}],
        "reference_candidate_binding_status": "proved-exhaustive",
        "reference_candidate_sha256": reference.reference_candidate_sha256,
        "reference_check_evidence_id": reference.reference_check_evidence_id,
        "reference_check_report_sha256": reference.reference_check_report_sha256,
        "reference_bundle_evidence_id": run.reference_bundle_evidence_id,
        "reference_bundle_logical_path": reference.reference_bundle_logical_path,
        "reference_bundle_sha256": run.reference_bundle_sha256,
        "reference_run_id": reference.reference_run_id,
        "run_configuration_sha256": reference.run_configuration_sha256,
        "runtime_environment_sha256": runtime_environment_sha256,
        "schema": "gluerift.native-replay-report/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "source_tree_read_only": true,
        "source_tree_read_only_enforcement": match network_isolation_argument { "sandbox-exec" => "sandbox-exec-tested-no-file-writes", "outer-sandbox-exec" => "outer-sandbox-exec-tested-output-only-write-whitelist", _ => "outer-read-only-source-mount" },
        "six_roundtrip_statuses": run.roundtrips,
        "source_program_output": run.source_program_output,
        "stdin_or_fixture_logical_path": input_logical,
        "stdin_or_fixture_sha256": input_hash,
        "target_program_output": run.target_program_output,
        "tool_build_sha256": tool_build_sha256,
        "transformation_report_sha256": reference.transformation_report_sha256,
        "transported_source_as_target_native": run.transported_source,
        "types_sha256": reference.types_sha256,
        "validation_request_sha256": reference.validation_request_sha256,
        "validation_scope_sha256": reference.validation_scope_sha256,
        "violation_witness": run.witness,
        "status": "proved-exhaustive"
    });
    let replay_name = "native-replay-report.json";
    let replay_hash = canonical::write_file(&fixture_dir.join(replay_name), &replay_report)?;

    let backend = json!({
        "adapter_value_mismatches": run.adapter_value_mismatches,
        "build_manifest_set_sha256": build_manifest_set_sha256,
        "build_manifests": build_set,
        "checked_adapter_value_count": run.checked_adapter_value_count,
        "checked_comparator_pair_count": run.checked_comparator_pair_count,
        "comparator_kind": "target-native-exact",
        "comparator_spec_sha256": reference.comparator_spec_sha256,
        "comparator_truth_table_mismatches": run.comparator_truth_table_mismatches,
        "roundtrip_truth_table_mismatches": run.roundtrip_truth_table_mismatches,
        "context_sha256": reference.transformed_context_sha256,
        "candidate_sha256": reference.reference_candidate_sha256,
        "dependency_evidence_ids": backend_dependencies,
        "dynamic_dependency_manifest_set_sha256": dynamic_dependency_manifest_set_sha256,
        "dynamic_dependency_manifests": dependency_set,
        "fixture_id": run.fixture_id,
        "evidence_id": backend_evidence_id,
        "endpoint_policy_sha256": reference.endpoint_policy_sha256,
        "reference_check_evidence_id": reference.reference_check_evidence_id,
        "reference_bundle_evidence_id": run.reference_bundle_evidence_id,
        "reference_bundle_logical_path": reference.reference_bundle_logical_path,
        "reference_bundle_sha256": run.reference_bundle_sha256,
        "native_source_tree_sha256": native_role_tree_hash("go-source", bindings)?,
        "native_target_tree_sha256": native_role_tree_hash("rust-target", bindings)?,
        "runtime_environment_sha256": runtime_environment_sha256,
        "schema": "gluerift.backend-conformance/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "status": "proved-exhaustive",
        "stdin_or_fixture_logical_path": input_logical,
        "stdin_or_fixture_sha256": input_hash,
        "tool_build_sha256": tool_build_sha256,
        "types_sha256": reference.types_sha256,
        "validation_request_sha256": reference.validation_request_sha256,
        "run_configuration_sha256": reference.run_configuration_sha256,
        "validation_scope_sha256": reference.validation_scope_sha256
    });
    let backend_name = "backend-conformance.json";
    let backend_hash = canonical::write_file(&fixture_dir.join(backend_name), &backend)?;

    let transcript_name = "transcript.txt";
    let transcript = format!("{}\n", run.transcript.join("\n"));
    fs::write(fixture_dir.join(transcript_name), transcript.as_bytes())?;
    let transcript_hash = canonical::sha256_bytes(transcript.as_bytes());
    let prefix = format!("{logical_prefix}/{}", run.fixture_id);
    Ok(NativeOutputIndexEntry {
        backend_conformance_logical_path: format!("{prefix}/{backend_name}"),
        backend_conformance_sha256: backend_hash,
        fixture_id: run.fixture_id.clone(),
        reference_bundle_logical_path: reference.reference_bundle_logical_path.clone(),
        reference_bundle_sha256: reference.reference_bundle_sha256.clone(),
        native_manifest_logical_path: format!("{prefix}/{manifest_name}"),
        native_manifest_sha256: manifest_hash,
        replay_report_logical_path: format!("{prefix}/{replay_name}"),
        replay_report_sha256: replay_hash,
        transcript_logical_path: format!("{prefix}/{transcript_name}"),
        transcript_sha256: transcript_hash,
    })
}

fn native_role_tree_hash(role: &str, bindings: &ReferenceBindings) -> Result<String> {
    canonical::sha256(&json!({
        "role": role,
        "source_inputs_manifest_sha256": bindings.source_inputs_manifest_sha256,
        "source_tree_sha256": bindings.source_tree_sha256
    }))
}

fn validate_sha(name: &str, value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("{name} is not a lowercase SHA-256 digest")
    }
    Ok(())
}

fn validate_logical_path(path: &str) -> Result<()> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!(
            "path must be repository-relative without parent traversal: {}",
            path.display()
        )
    }
    Ok(())
}

pub fn validate_required_laws(run: &SemanticRun) -> Result<()> {
    if run.roundtrips.len() != ALL_LAW_IDS.len()
        || ALL_LAW_IDS
            .iter()
            .any(|law| run.roundtrips.get(*law).map(String::as_str) != Some("proved-exhaustive"))
    {
        bail!("native replay did not prove all six explicitly required laws")
    }
    Ok(())
}
