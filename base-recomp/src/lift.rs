//! Minimal x86-32 static lifter (subset). Capstone full decode lands in R1+.

use crate::sir::{BasicBlock, Function, Module, Op, VReg};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiftError {
    #[error("empty input")]
    Empty,
}

/// Lift a flat x86-32 instruction stream into one function (linear block).
///
/// Supported (R0/R1 subset):
/// - `90` nop
/// - `C3` ret
/// - `B8 iv` mov eax, imm32
/// - `05 iv` add eax, imm32
pub fn lift_x86_32(bytes: &[u8], name: &str) -> Result<Module, LiftError> {
    if bytes.is_empty() {
        return Err(LiftError::Empty);
    }

    let eax = VReg(0);
    let mut ops = Vec::new();
    let mut i = 0usize;
    let mut gaps = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            0x90 => {
                ops.push(Op::Nop);
                i += 1;
            }
            0xC3 => {
                ops.push(Op::Ret);
                i += 1;
            }
            0xB8 if i + 5 <= bytes.len() => {
                let imm = u32::from_le_bytes(bytes[i + 1..i + 5].try_into().unwrap());
                ops.push(Op::MovImm { dst: eax, imm });
                i += 5;
            }
            0x05 if i + 5 <= bytes.len() => {
                let imm = u32::from_le_bytes(bytes[i + 1..i + 5].try_into().unwrap());
                ops.push(Op::AddImm { dst: eax, imm });
                i += 5;
            }
            other => {
                gaps += 1;
                ops.push(Op::Unknown {
                    offset: i as u64,
                    bytes: vec![other],
                    note: format!("unsupported opcode 0x{other:02x}"),
                });
                i += 1;
            }
        }
    }

    Ok(Module {
        name: name.to_string(),
        source_isa: "x86_32".into(),
        functions: vec![Function {
            name: name.to_string(),
            blocks: vec![BasicBlock {
                label: "entry".into(),
                ops,
            }],
        }],
        lift_gaps: gaps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lift_nop_ret() {
        let m = lift_x86_32(&[0x90, 0xC3], "f").unwrap();
        assert_eq!(m.lift_gaps, 0);
        assert_eq!(m.functions[0].blocks[0].ops.len(), 2);
    }

    #[test]
    fn lift_mov_add_ret() {
        // mov eax, 1 ; add eax, 2 ; ret
        let bytes = [
            0xB8, 0x01, 0x00, 0x00, 0x00, 0x05, 0x02, 0x00, 0x00, 0x00, 0xC3,
        ];
        let m = lift_x86_32(&bytes, "add1").unwrap();
        assert_eq!(m.lift_gaps, 0);
        assert!(matches!(
            m.functions[0].blocks[0].ops[0],
            Op::MovImm { imm: 1, .. }
        ));
    }

    #[test]
    fn unknown_opcode_is_gap() {
        let m = lift_x86_32(&[0xCC], "int3").unwrap();
        assert_eq!(m.lift_gaps, 1);
        assert_eq!(m.count_gaps(), 1);
    }
}
