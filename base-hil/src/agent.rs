/// Host Agent — template EXPERIMENTAL. Sem probe físico, só simulação.
use std::path::Path;

use crate::probe::ProbeFirmware;

/// Presença de hardware. Flash real só com [`ProbePresence::Detected`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbePresence {
    /// Sem USB/CMSIS-DAP — default do CI e de `connect`.
    Simulated,
    /// Probe reconhecido (ainda não implementado em host).
    Detected,
}

/// Representa uma amostra capturada pelo probe
#[derive(Debug, Clone)]
pub struct HilSample {
    pub timestamp_ns: u64,
    pub address: u16,
    pub data: u8,
    pub flags: u8,
}

/// Agente host que se comunica com o probe HIL
pub struct HilAgent {
    presence: ProbePresence,
}

impl HilAgent {
    /// Abre canal com o probe. Sem device no host ⇒ sempre [`ProbePresence::Simulated`].
    pub fn connect(vid: u16, pid: u16) -> Result<Self, String> {
        tracing::info!(
            "[HIL][EXPERIMENTAL] Connecting to probe {:04x}:{:04x} (simulated; no USB enumerate)",
            vid,
            pid
        );
        Ok(Self {
            presence: ProbePresence::Simulated,
        })
    }

    /// Construtor de teste / futuro path com probe real.
    pub fn with_presence(presence: ProbePresence) -> Self {
        Self { presence }
    }

    pub fn presence(&self) -> ProbePresence {
        self.presence
    }

    pub fn can_flash(&self) -> bool {
        matches!(self.presence, ProbePresence::Detected)
    }

    /// Flash **só** com probe detectado. Em modo simulado falha de propósito (S4).
    pub fn flash_probe_firmware(&self, _image: &[u8]) -> Result<(), String> {
        if !self.can_flash() {
            return Err(
                "HIL EXPERIMENTAL: flash requer ProbePresence::Detected (CMSIS-DAP/open probe); \
                 connect() atual é simulado e não grava silício"
                    .into(),
            );
        }
        Err("HIL EXPERIMENTAL: path Detected ainda não implementa programador".into())
    }

    /// Lê amostras do probe (modo simulado)
    pub fn read_samples(&self, count: usize) -> Vec<HilSample> {
        (0..count)
            .map(|i| HilSample {
                timestamp_ns: i as u64 * 1000,
                address: 0x1000 + i as u16,
                data: (i & 0xFF) as u8,
                flags: 0,
            })
            .collect()
    }

    /// Converte amostras para DeviceTrace (formato do base-check)
    pub fn samples_to_trace(samples: &[HilSample]) -> base_check::tracer::DeviceTrace {
        let events: Vec<_> = samples
            .iter()
            .map(|s| base_check::tracer::TraceEvent {
                timestamp_ns: s.timestamp_ns,
                channel: format!("BUS_{:04x}", s.address),
                event_type: base_check::tracer::EventType::MmioWrite,
                address: s.address as u64,
                value: Some(s.data as u64),
            })
            .collect();

        base_check::tracer::DeviceTrace {
            source: "HIL Probe [EXPERIMENTAL]".into(),
            device_name: "HIL Capture".into(),
            events,
        }
    }

    /// Exporta amostras como CSV no formato Saleae
    pub fn export_csv(samples: &[HilSample], path: &Path) -> Result<(), std::io::Error> {
        let mut csv = String::from("Time[s],Channel,Type,Data\n");
        for s in samples {
            csv.push_str(&format!(
                "{:.9},BUS_{:04x},WRITE,0x{:04x}=0x{:02x}\n",
                s.timestamp_ns as f64 / 1_000_000_000.0,
                s.address,
                s.address,
                s.data
            ));
        }
        std::fs::write(path, csv)
    }

    /// Script de scaffold do projeto embutido (não chama CLI — `base hil` ainda não existe).
    pub fn generate_build_script() -> String {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# B.A.S.E. HIL Probe — EXPERIMENTAL scaffold\n");
        script.push_str("# Não faz flash. Não faz parte do `base pipeline` default.\n\n");
        script.push_str("set -euo pipefail\n\n");
        script.push_str("PROBE_DIR=\"hil_probe\"\n");
        script.push_str("mkdir -p \"$PROBE_DIR/src\"\n\n");
        script.push_str("echo \"Escreva o stub com a lib host:\"\n");
        script.push_str("echo \"  use base_hil::probe::ProbeFirmware;\"\n");
        script.push_str("echo \"  std::fs::write(\\\"$PROBE_DIR/src/main.rs\\\", ProbeFirmware::generate());\"\n\n");
        script.push_str("cat > \"$PROBE_DIR/Cargo.toml\" << 'EOF'\n");
        script.push_str("[package]\n");
        script.push_str("name = \"hil-probe\"\n");
        script.push_str("version = \"0.1.0\"\n");
        script.push_str("edition = \"2021\"\n\n");
        script.push_str("# Dependências de target embutido — fora do CI default do workspace.\n");
        script.push_str("[dependencies]\n");
        script.push_str("rp235x-hal = { git = \"https://github.com/rp-rs/rp-hal\" }\n");
        script.push_str("usb-device = \"0.3\"\n");
        script.push_str("usbd-serial = \"0.2\"\n");
        script.push_str("panic-halt = \"0.2\"\n");
        script.push_str("cortex-m-rt = \"0.7\"\n");
        script.push_str("cortex-m = \"0.7\"\n");
        script.push_str("EOF\n\n");
        script.push_str("echo \"[EXPERIMENTAL] scaffold em $PROBE_DIR/\"\n");
        script.push_str("echo \"Build (manual, precisa target): cargo build --release --target thumbv8m.main-none-eabi\"\n");
        script.push_str("echo \"Flash: só com probe Detected — HilAgent::flash_probe_firmware\"\n");
        script
    }

    /// Escreve o stub de firmware gerado (host) para um path.
    pub fn write_probe_stub(path: &Path) -> Result<(), std::io::Error> {
        std::fs::write(path, ProbeFirmware::generate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_agent_connect_is_simulated() {
        let agent = HilAgent::connect(0xCAFE, 0x4007).unwrap();
        assert_eq!(agent.presence(), ProbePresence::Simulated);
        assert!(!agent.can_flash());
    }

    #[test]
    fn test_flash_denied_without_probe() {
        let agent = HilAgent::connect(0xCAFE, 0x4007).unwrap();
        let err = agent.flash_probe_firmware(&[0u8; 4]).unwrap_err();
        assert!(err.contains("EXPERIMENTAL"));
        assert!(err.contains("Detected"));
    }

    #[test]
    fn test_flash_denied_even_when_marked_detected_until_programmer_exists() {
        let agent = HilAgent::with_presence(ProbePresence::Detected);
        assert!(agent.can_flash());
        let err = agent.flash_probe_firmware(&[0u8; 4]).unwrap_err();
        assert!(err.contains("não implementa"));
    }

    #[test]
    fn test_read_samples() {
        let agent = HilAgent::connect(0xCAFE, 0x4007).unwrap();
        let samples = agent.read_samples(10);
        assert_eq!(samples.len(), 10);
        assert_eq!(samples[0].address, 0x1000);
    }

    #[test]
    fn test_samples_to_trace() {
        let agent = HilAgent::connect(0xCAFE, 0x4007).unwrap();
        let samples = agent.read_samples(5);
        let trace = HilAgent::samples_to_trace(&samples);
        assert_eq!(trace.events.len(), 5);
        assert!(trace.source.contains("HIL"));
        assert!(trace.source.contains("EXPERIMENTAL"));
    }

    #[test]
    fn test_export_csv() {
        let agent = HilAgent::connect(0xCAFE, 0x4007).unwrap();
        let samples = agent.read_samples(3);
        let dir = tempdir().unwrap();
        let path = dir.path().join("capture.csv");
        HilAgent::export_csv(&samples, &path).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("BUS_1000"));
    }

    #[test]
    fn test_build_script_no_fake_cli() {
        let script = HilAgent::generate_build_script();
        assert!(script.contains("hil-probe"));
        assert!(script.contains("EXPERIMENTAL"));
        assert!(!script.contains("base hil "));
        assert!(script.contains("thumbv8m"));
    }

    #[test]
    fn test_write_probe_stub() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("main.rs");
        HilAgent::write_probe_stub(&path).unwrap();
        let fw = std::fs::read_to_string(&path).unwrap();
        assert!(fw.contains("HIL Probe"));
    }
}
