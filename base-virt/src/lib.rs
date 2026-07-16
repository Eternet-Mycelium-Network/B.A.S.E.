//! Specter Live — VM comportamental (QEMU primário).
//!
//! Ingest NDJSON MMIO/IRQ → [`EvidenceDb`] → Ψ em janelas.
//! ≠ OS turnkey · ≠ HIL production · `generates_os=false`.

pub mod live;
pub mod qemu;
pub mod session;
pub mod trace;

pub use live::{run_live_windows, LiveConfig, LiveWindowScore};
pub use qemu::{launch_qemu, resolve_qemu_bin, QemuLaunchOpts, QemuLaunchResult};
pub use session::{VirtSessionReport, VirtSessionWindow};
pub use trace::{
    ingest_ndjson, ingest_ndjson_path, parse_ndjson_line, TraceEvent, TraceSourceError,
};
