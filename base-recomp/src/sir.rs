//! Static Intermediate Representation (SIR).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VReg(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    Nop,
    Ret,
    /// `dst := imm` (32-bit for now).
    MovImm { dst: VReg, imm: u32 },
    /// `dst := dst + imm`
    AddImm { dst: VReg, imm: u32 },
    /// Unliftable / unsupported opcode — wedge for `base-reason`.
    Unknown { offset: u64, bytes: Vec<u8>, note: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicBlock {
    pub label: String,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub source_isa: String,
    pub functions: Vec<Function>,
    pub lift_gaps: usize,
}

impl Module {
    pub fn count_gaps(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|f| f.blocks.iter())
            .flat_map(|b| b.ops.iter())
            .filter(|o| matches!(o, Op::Unknown { .. }))
            .count()
    }
}
