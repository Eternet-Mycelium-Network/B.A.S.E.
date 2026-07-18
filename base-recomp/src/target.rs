//! Target ISA enumeration (amd64 ≡ x86_64).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetIsa {
    /// x86_64 / **amd64** (same backend).
    X86_64,
    Arm,
    AArch64,
    Mips,
    Ppc,
    Sparc,
    /// SuperH / Hitachi — SH-2 (Saturn) · SH-4 (Dreamcast) flavor via [`SuperHFlavor`].
    SuperH(SuperHFlavor),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SuperHFlavor {
    #[default]
    Sh2,
    Sh4,
}

#[derive(Debug, Error)]
#[error("unknown target ISA: {0} (try x86_64|amd64|arm|arm64|mips|ppc|sparc|sh2|sh4)")]
pub struct ParseTargetError(pub String);

impl TargetIsa {
    pub fn all_canonical() -> &'static [TargetIsa] {
        &[
            TargetIsa::X86_64,
            TargetIsa::Arm,
            TargetIsa::AArch64,
            TargetIsa::Mips,
            TargetIsa::Ppc,
            TargetIsa::Sparc,
            TargetIsa::SuperH(SuperHFlavor::Sh2),
            TargetIsa::SuperH(SuperHFlavor::Sh4),
        ]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TargetIsa::X86_64 => "x86_64",
            TargetIsa::Arm => "arm",
            TargetIsa::AArch64 => "aarch64",
            TargetIsa::Mips => "mips",
            TargetIsa::Ppc => "ppc",
            TargetIsa::Sparc => "sparc",
            TargetIsa::SuperH(SuperHFlavor::Sh2) => "sh2",
            TargetIsa::SuperH(SuperHFlavor::Sh4) => "sh4",
        }
    }
}

impl fmt::Display for TargetIsa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TargetIsa {
    type Err = ParseTargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = s.trim().to_ascii_lowercase();
        Ok(match key.as_str() {
            "x86_64" | "x86-64" | "amd64" | "x64" => TargetIsa::X86_64,
            "arm" | "arm32" | "aarch32" => TargetIsa::Arm,
            "arm64" | "aarch64" => TargetIsa::AArch64,
            "mips" | "mips32" => TargetIsa::Mips,
            "ppc" | "powerpc" | "ppc32" => TargetIsa::Ppc,
            "sparc" | "sparc32" => TargetIsa::Sparc,
            "sh" | "superh" | "sh2" | "hitachi" => TargetIsa::SuperH(SuperHFlavor::Sh2),
            "sh4" => TargetIsa::SuperH(SuperHFlavor::Sh4),
            other => return Err(ParseTargetError(other.to_string())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amd64_aliases_x86_64() {
        assert_eq!("amd64".parse::<TargetIsa>().unwrap(), TargetIsa::X86_64);
        assert_eq!("x86_64".parse::<TargetIsa>().unwrap(), TargetIsa::X86_64);
    }

    #[test]
    fn superh_flavors() {
        assert_eq!(
            "sh2".parse::<TargetIsa>().unwrap(),
            TargetIsa::SuperH(SuperHFlavor::Sh2)
        );
        assert_eq!(
            "sh4".parse::<TargetIsa>().unwrap(),
            TargetIsa::SuperH(SuperHFlavor::Sh4)
        );
    }

    #[test]
    fn all_canonical_len() {
        assert_eq!(TargetIsa::all_canonical().len(), 8);
    }
}
