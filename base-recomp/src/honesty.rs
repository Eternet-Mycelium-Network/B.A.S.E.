//! Honesty gates for static recompilation.

use base_core::honesty::{AUTO_FIX_COMPLETE, GENERATES_OS};

/// Pipeline still incomplete as a product — always false through v1.7 R5 gate.
pub const STATIC_RECOMP_COMPLETE: bool = false;

/// Win32 / PE imports / SEH not in v1.7 north star.
pub const WIN32_ABI_COMPLETE: bool = false;

/// Never claim arbitrary PE execution.
pub const RUNS_ANY_PE: bool = false;

pub const BANNER: &str = "≠ Wine / ≠ Win32 completo: `static_recomp_complete: false` · `win32_abi_complete: false` · `runs_any_pe: false` — lift estático SIR ≠ correr .exe arbitrário.";

pub fn markdown_section() -> String {
    format!(
        "## Honesty (static recomp)\n\n- {}\n- `generates_os: {}` · `auto_fix_complete: {}`\n- `static_recomp_complete: {}` · `win32_abi_complete: {}` · `runs_any_pe: {}`\n",
        BANNER,
        GENERATES_OS,
        AUTO_FIX_COMPLETE,
        STATIC_RECOMP_COMPLETE,
        WIN32_ABI_COMPLETE,
        RUNS_ANY_PE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honesty_flags_stay_false() {
        assert!(!STATIC_RECOMP_COMPLETE);
        assert!(!WIN32_ABI_COMPLETE);
        assert!(!RUNS_ANY_PE);
        assert!(!GENERATES_OS);
    }
}
