use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Docker,
    Direct,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub runtime: String,
    pub pid: u32,
    pub started_at_unix_ms: u64,
    pub mode: ExecutionMode,
    pub log_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeProcessStatus {
    pub runtime: String,
    pub pid: Option<u32>,
    pub running: bool,
    pub mode: ExecutionMode,
    pub log_path: PathBuf,
}

pub struct ProcessManager {
    mode: ExecutionMode,
    state_dir: PathBuf,
    log_dir: PathBuf,
}

impl ProcessManager {
    pub fn new(mode: ExecutionMode) -> Result<Self> {
        let root = clawden_root_dir()?;
        let state_dir = root.join("run");
        let log_dir = root.join("logs");
        fs::create_dir_all(&state_dir)?;
        fs::create_dir_all(&log_dir)?;
        Ok(Self {
            mode,
            state_dir,
            log_dir,
        })
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    pub fn resolve_mode(&self, force_no_docker: bool) -> ExecutionMode {
        if force_no_docker {
            return ExecutionMode::Direct;
        }

        match self.mode {
            ExecutionMode::Auto => {
                if Self::docker_available() {
                    ExecutionMode::Docker
                } else {
                    ExecutionMode::Direct
                }
            }
            explicit => explicit,
        }
    }

    pub fn start_direct(&self, runtime: &str, executable: &Path, args: &[String]) -> Result<ProcessInfo> {
        if !executable.exists() {
            return Err(anyhow!(
                "runtime executable not found: {}",
                executable.display()
            ));
        }

        let log_path = self.log_dir.join(format!("{runtime}.log"));
        let stdout = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("opening runtime log file {}", log_path.display()))?;
        let stderr = stdout.try_clone()?;

        let mut command = Command::new(executable);
        command.args(args);
        command.stdout(Stdio::from(stdout));
        command.stderr(Stdio::from(stderr));

        let child = command
            .spawn()
            .with_context(|| format!("failed to spawn {}", executable.display()))?;

        let info = ProcessInfo {
            runtime: runtime.to_string(),
            pid: child.id(),
            started_at_unix_ms: now_ms(),
            mode: ExecutionMode::Direct,
            log_path: log_path.clone(),
        };

        self.write_pid_file(runtime, &info)?;
        Ok(info)
    }

    pub fn stop(&self, runtime: &str) -> Result<()> {
        let Some(info) = self.read_pid_file(runtime)? else {
            return Ok(());
        };

        let pid = info.pid.to_string();
        let _ = Command::new("kill").args(["-TERM", &pid]).status();
        for _ in 0..20 {
            if !is_pid_running(info.pid) {
                self.remove_pid_file(runtime)?;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }

        let _ = Command::new("kill").args(["-KILL", &pid]).status();
        self.remove_pid_file(runtime)?;
        Ok(())
    }

    pub fn list_statuses(&self) -> Result<Vec<RuntimeProcessStatus>> {
        let mut statuses = Vec::new();
        if !self.state_dir.exists() {
            return Ok(statuses);
        }

        for entry in fs::read_dir(&self.state_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("pid") {
                continue;
            }

            let runtime = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            if let Some(info) = self.read_pid_file(&runtime)? {
                statuses.push(RuntimeProcessStatus {
                    runtime,
                    pid: Some(info.pid),
                    running: is_pid_running(info.pid),
                    mode: info.mode,
                    log_path: info.log_path,
                });
            }
        }

        statuses.sort_by(|a, b| a.runtime.cmp(&b.runtime));
        Ok(statuses)
    }

    pub fn tail_logs(&self, runtime: &str, lines: usize) -> Result<String> {
        let log_path = self.log_dir.join(format!("{runtime}.log"));
        if !log_path.exists() {
            return Ok(String::new());
        }
        let content = fs::read_to_string(&log_path)?;
        let rows: Vec<&str> = content.lines().collect();
        let start = rows.len().saturating_sub(lines);
        Ok(rows[start..].join("\n"))
    }

    fn write_pid_file(&self, runtime: &str, info: &ProcessInfo) -> Result<()> {
        let path = self.pid_file(runtime);
        let body = serde_json::to_string_pretty(info)?;
        let mut file = File::create(&path)?;
        file.write_all(body.as_bytes())?;
        Ok(())
    }

    fn read_pid_file(&self, runtime: &str) -> Result<Option<ProcessInfo>> {
        let path = self.pid_file(runtime);
        if !path.exists() {
            return Ok(None);
        }
        let body = fs::read_to_string(path)?;
        let info: ProcessInfo = serde_json::from_str(&body)?;
        Ok(Some(info))
    }

    fn remove_pid_file(&self, runtime: &str) -> Result<()> {
        let path = self.pid_file(runtime);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn pid_file(&self, runtime: &str) -> PathBuf {
        self.state_dir.join(format!("{runtime}.pid"))
    }
}

fn is_pid_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX_EPOCH")
        .as_millis() as u64
}

fn clawden_root_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(".clawden"))
}
