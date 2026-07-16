//! Lançador QEMU mínimo (opt-in) — smoke + hook para Specter Live.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuLaunchResult {
    pub launched: bool,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub bin: String,
    pub kernel: Option<String>,
    pub log_path: Option<String>,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone)]
pub struct QemuLaunchOpts {
    pub bin: String,
    pub machine: String,
    pub cpu: String,
    pub memory: String,
    pub kernel: Option<PathBuf>,
    pub timeout_sec: u64,
    pub log_path: PathBuf,
    pub extra_args: Vec<String>,
}

impl Default for QemuLaunchOpts {
    fn default() -> Self {
        Self {
            bin: "qemu-system-aarch64".into(),
            machine: "virt".into(),
            cpu: "cortex-a72".into(),
            memory: "256M".into(),
            kernel: None,
            timeout_sec: 8,
            log_path: PathBuf::from("qemu.log"),
            extra_args: Vec::new(),
        }
    }
}

fn which_bin(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Corre QEMU com timeout; não exige guest saudável (smoke / live seed).
pub fn launch_qemu(opts: &QemuLaunchOpts) -> anyhow::Result<QemuLaunchResult> {
    if !which_bin(&opts.bin) {
        return Ok(QemuLaunchResult {
            launched: false,
            skipped: true,
            skip_reason: Some(format!("{} not installed", opts.bin)),
            exit_code: None,
            timed_out: false,
            bin: opts.bin.clone(),
            kernel: opts.kernel.as_ref().map(|p| p.display().to_string()),
            log_path: None,
            timeout_sec: opts.timeout_sec,
        });
    }

    let kernel = opts
        .kernel
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("QEMU launch requires --kernel"))?;

    if let Some(parent) = opts.log_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let log = std::fs::File::create(&opts.log_path)?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(&opts.bin);
    cmd.arg("-machine")
        .arg(&opts.machine)
        .arg("-cpu")
        .arg(&opts.cpu)
        .arg("-m")
        .arg(&opts.memory)
        .arg("-nographic")
        .arg("-kernel")
        .arg(kernel)
        .args(&opts.extra_args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));

    let mut child = cmd.spawn()?;
    let timeout = Duration::from_secs(opts.timeout_sec.max(1));
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let exit_code = loop {
        match child.try_wait()? {
            Some(status) => break status.code(),
            None => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break Some(124);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    };

    Ok(QemuLaunchResult {
        launched: true,
        skipped: false,
        skip_reason: None,
        exit_code,
        timed_out,
        bin: opts.bin.clone(),
        kernel: Some(kernel.display().to_string()),
        log_path: Some(opts.log_path.display().to_string()),
        timeout_sec: opts.timeout_sec,
    })
}

/// Resolve binário QEMU a partir do path opcional.
pub fn resolve_qemu_bin(explicit: Option<&Path>, default: &str) -> String {
    explicit
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_qemu_skips() {
        let opts = QemuLaunchOpts {
            bin: "qemu-system-base-virt-missing-xyz".into(),
            kernel: Some(PathBuf::from("/tmp/nope.bin")),
            ..Default::default()
        };
        let r = launch_qemu(&opts).unwrap();
        assert!(r.skipped);
        assert!(!r.launched);
    }
}
