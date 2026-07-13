/// Host Agent — conecta-se ao probe RP2350 e gerencia captura.
use std::path::Path;

/// Representa uma amostra capturada pelo probe
#[derive(Debug, Clone)]
pub struct HilSample {
    pub timestamp_ns: u64,
    pub address: u16,
    pub data: u8,
    pub flags: u8,
}

/// Agente host que se comunica com o probe HIL
pub struct HilAgent;

impl HilAgent {
    /// Conecta a um probe via USB (usando PID/VID)
    pub fn connect(vid: u16, pid: u16) -> Result<Self, String> {
        tracing::info!("[HIL] Connecting to probe {:04x}:{:04x}...", vid, pid);
        // Em modo simulado, retorna o agente sem conexão real
        Ok(Self)
    }

    /// Lê amostras do probe (modo simulado)
    pub fn read_samples(&self, count: usize) -> Vec<HilSample> {
        // Simula captura de amostras para teste
        (0..count).map(|i| HilSample {
            timestamp_ns: i as u64 * 1000,
            address: 0x1000 + i as u16,
            data: (i & 0xFF) as u8,
            flags: 0,
        }).collect()
    }

    /// Converte amostras para DeviceTrace (formato do base-check)
    pub fn samples_to_trace(samples: &[HilSample]) -> base_check::tracer::DeviceTrace {
        let events: Vec<_> = samples.iter().map(|s| base_check::tracer::TraceEvent {
            timestamp_ns: s.timestamp_ns,
            channel: format!("BUS_{:04x}", s.address),
            event_type: base_check::tracer::EventType::MmioWrite,
            address: s.address as u64,
            value: Some(s.data as u64),
        }).collect();

        base_check::tracer::DeviceTrace {
            source: "HIL Probe".into(),
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
                s.address, s.address, s.data
            ));
        }
        std::fs::write(path, csv)
    }

    /// Gera um script de build para o firmware do probe
    pub fn generate_build_script() -> String {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# B.A.S.E. HIL Probe Build Script\n\n");
        script.push_str("set -e\n\n");
        script.push_str("PROBE_DIR=\"hil_probe\"\n");
        script.push_str("mkdir -p $PROBE_DIR/src\n\n");
        script.push_str("# Gerar firmware\n");
        script.push_str("base hil probe-fw > $PROBE_DIR/src/main.rs\n\n");
        script.push_str("# Criar Cargo.toml\n");
        script.push_str("cat > $PROBE_DIR/Cargo.toml << 'EOF'\n");
        script.push_str("[package]\n");
        script.push_str("name = \"hil-probe\"\n");
        script.push_str("version = \"0.1.0\"\n");
        script.push_str("edition = \"2021\"\n\n");
        script.push_str("[dependencies]\n");
        script.push_str("rp235x-hal = { git = \"https://github.com/rp-rs/rp-hal\" }\n");
        script.push_str("usb-device = \"0.3\"\n");
        script.push_str("usbd-serial = \"0.2\"\n");
        script.push_str("panic-halt = \"0.2\"\n");
        script.push_str("cortex-m-rt = \"0.7\"\n");
        script.push_str("cortex-m = \"0.7\"\n");
        script.push_str("EOF\n\n");
        script.push_str("echo \"Probe firmware generated in $PROBE_DIR/\"\n");
        script.push_str("echo \"Build with: cargo build --release --target thumbv8m.main-none-eabi\"\n");
        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_agent_connect() {
        let agent = HilAgent::connect(0xCAFE, 0x4007);
        assert!(agent.is_ok());
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
    fn test_build_script() {
        let script = HilAgent::generate_build_script();
        assert!(script.contains("hil-probe"));
        assert!(script.contains("rp235x-hal"));
        assert!(script.contains("thumbv8m"));
    }
}
