//! HIL Cluster — **EXPERIMENTAL** template (host agent + gerador de stub de firmware).
//!
//! - Compila e testa no host **sem** hardware.
//! - Enumerate USB real: feature opt-in `hil_usb` (não no CI default).
//! - Não flashea silício sem [`agent::ProbePresence::Detected`].
//! - Não entra no `base pipeline` default.

pub mod agent;
pub mod probe;
mod usb;

pub use agent::{
    FlashDenied, FlashReceipt, HilAgent, HilSample, ProbePresence, DEFAULT_PROBE_PID,
    DEFAULT_PROBE_VID, ENV_MOCK_DETECTED,
};
pub use probe::ProbeFirmware;
