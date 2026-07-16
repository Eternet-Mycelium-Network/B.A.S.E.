//! Specter Live — VM comportamental (QEMU primário).
//!
//! Ingest NDJSON MMIO/IRQ → [`EvidenceDb`] → Ψ em janelas.
//! Plugin TCG (`plugin/`) + QMP (`qmp`) + Study↔Live (`study_live`) —
//! ≠ OS turnkey · ≠ HIL production.

pub mod live;
pub mod qemu;
pub mod qmp;
pub mod session;
pub mod study_live;
pub mod trace;

pub use live::{run_live_windows, LiveConfig, LiveWindowScore};
pub use qemu::{
    format_plugin_cli, launch_qemu, resolve_qemu_bin, spawn_qemu_live, QemuLaunchOpts,
    QemuLaunchResult, QemuLiveSession,
};
pub use qmp::{probe_session, QmpClient, QmpError};
pub use session::{VirtSessionReport, VirtSessionWindow};
pub use study_live::{load_evidence_flexible, run_live_study, LiveStudyReport};
pub use trace::{
    ingest_ndjson, ingest_ndjson_path, parse_ndjson_line, TraceEvent, TraceSourceError,
};
