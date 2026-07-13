/// HIL Cluster — hardware probe firmware template + host agent.
///
/// O probe é um RP2350 com firmware Rust (rp-hal) que captura
/// barramentos paralelos via PIO e envia traces USB para o base-check.
pub mod probe;
pub mod agent;

pub use agent::HilAgent;
