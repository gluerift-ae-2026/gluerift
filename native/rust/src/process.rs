use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

// The contract requires an explicit deterministic limit, not a benchmark
// threshold.  Thirty seconds keeps a cold dyld/sandbox launch from becoming a
// flaky semantic failure while remaining a strict per-operation bound.
pub const TIMEOUT: Duration = Duration::from_secs(30);
pub const STDIN_LIMIT: usize = 4096;
pub const STDOUT_LIMIT: usize = 65_536;
pub const STDERR_LIMIT: usize = 65_536;

#[derive(Clone, Debug, Serialize)]
pub struct ProcessResult {
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub environment_mode: String,
    pub exit_code: i32,
    pub network_isolation: String,
    pub operation_id: String,
    pub role: String,
    pub source_tree_read_only_enforcement: String,
    pub stderr: Vec<u8>,
    pub stdout: Vec<u8>,
    pub timed_out: bool,
    pub working_directory: String,
}

pub fn fixed_environment() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("LANG".to_owned(), "C.UTF-8".to_owned()),
        ("LC_ALL".to_owned(), "C.UTF-8".to_owned()),
        ("SOURCE_DATE_EPOCH".to_owned(), "1767225600".to_owned()),
        ("TZ".to_owned(), "UTC".to_owned()),
    ])
}

fn sandbox_profile(_repository: &Path) -> Result<String> {
    Ok("(version 1)(allow default)(deny network*)(deny file-write* (require-all (require-not (literal \"/dev/null\")) (require-not (literal \"/dev/urandom\"))))".to_owned())
}

pub fn verify_source_read_only(repository: &Path) -> Result<()> {
    if !Path::new("/usr/bin/sandbox-exec").is_file() {
        bail!("sandbox-exec is unavailable")
    }
    let probe = Path::new("/private/tmp").join(format!(
        "gluerift-inner-runtime-deny-probe.{}",
        std::process::id()
    ));
    if probe.exists() {
        bail!("sandbox write-probe path already exists")
    }
    let status = Command::new("/usr/bin/sandbox-exec")
        .arg("-p")
        .arg(sandbox_profile(repository)?)
        .arg("/usr/bin/touch")
        .arg(&probe)
        .env_clear()
        .envs(fixed_environment())
        .status()
        .context("execute source-tree write denial probe")?;
    let created = probe.exists();
    if status.success() || created {
        bail!("source-tree write denial probe was not blocked")
    }
    Ok(())
}

pub fn run(
    executable: &Path,
    logical_executable: &str,
    role: &str,
    operation_id: &str,
    arguments: &[String],
    input: &[u8],
    repository: &Path,
    cwd: &Path,
    logical_cwd: &str,
    use_sandbox_exec: bool,
    outer_isolation: &str,
) -> Result<ProcessResult> {
    if input.len() > STDIN_LIMIT {
        bail!("input exceeds {STDIN_LIMIT} bytes")
    }
    let environment = fixed_environment();
    let mut command;
    let mut canonical_argv = vec![logical_executable.to_owned()];
    canonical_argv.extend_from_slice(arguments);
    if use_sandbox_exec {
        if !Path::new("/usr/bin/sandbox-exec").is_file() {
            bail!("sandbox-exec requested but unavailable")
        }
        command = Command::new("/usr/bin/sandbox-exec");
        command
            .arg("-p")
            .arg(sandbox_profile(repository)?)
            .arg(executable);
    } else {
        command = Command::new(executable);
    }
    command.args(arguments);
    command
        .current_dir(cwd)
        .env_clear()
        .envs(&environment)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("spawn {}", executable.display()))?;
    let mut stdin = child.stdin.take().context("capture child stdin")?;
    let stdout = child.stdout.take().context("capture child stdout")?;
    let stderr = child.stderr.take().context("capture child stderr")?;
    let input = input.to_vec();
    let input_thread = thread::spawn(move || -> std::io::Result<()> {
        stdin.write_all(&input)?;
        drop(stdin);
        Ok(())
    });
    let stdout_thread = thread::spawn(move || read_bounded(stdout, STDOUT_LIMIT));
    let stderr_thread = thread::spawn(move || read_bounded(stderr, STDERR_LIMIT));

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= TIMEOUT {
            timed_out = true;
            child.kill().context("kill timed-out native process")?;
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(5));
    };
    input_thread
        .join()
        .map_err(|_| anyhow::anyhow!("stdin thread panicked"))??;
    let stdout = stdout_thread
        .join()
        .map_err(|_| anyhow::anyhow!("stdout thread panicked"))??;
    let stderr = stderr_thread
        .join()
        .map_err(|_| anyhow::anyhow!("stderr thread panicked"))??;
    if timed_out {
        bail!("native process {role}/{operation_id} timed out")
    }
    let exit_code = status.code().unwrap_or(-1);
    if exit_code != 0 {
        bail!(
            "native process {role}/{operation_id} exited {exit_code}: {}",
            String::from_utf8_lossy(&stderr)
        )
    }
    Ok(ProcessResult {
        argv: canonical_argv,
        environment,
        environment_mode: "empty-plus-whitelist".to_owned(),
        exit_code,
        network_isolation: match (use_sandbox_exec, outer_isolation) {
            (true, _) => "sandbox-exec-deny-network",
            (false, "outer-sandbox-exec") => "outer-sandbox-exec-deny-network",
            (false, _) => "outer-network-disabled-context",
        }
        .to_owned(),
        operation_id: operation_id.to_owned(),
        role: role.to_owned(),
        source_tree_read_only_enforcement: match (use_sandbox_exec, outer_isolation) {
            (true, _) => "sandbox-exec-no-file-writes",
            (false, "outer-sandbox-exec") => "outer-sandbox-exec-output-only-write-whitelist",
            (false, _) => "outer-read-only-source-mount",
        }
        .to_owned(),
        stderr,
        stdout,
        timed_out,
        working_directory: logical_cwd.to_owned(),
    })
}

fn read_bounded<R: Read>(reader: R, limit: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    reader.take((limit + 1) as u64).read_to_end(&mut output)?;
    if output.len() > limit {
        bail!("native process output exceeds {limit} bytes")
    }
    Ok(output)
}
