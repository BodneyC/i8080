//! 8080 disassembler
//!
//! This is much simpler than the assembler, it reads a stream of `u8`s from a file and interprets
//! them as 8080 instructions.
//!
//! This does nothing to try to "intelligently" resolve labels, macros, strings-literals, etc. For
//! example,
//!
//! ```asm
//! DB 'hello world'
//! ```
//!
//! would be disassembled as:
//!
//! ```asm
//! MOV L, B
//! MOV H, L
//! MOV L, H
//! MOV L, H
//! MOV L, A
//! ---
//! MOV M, A
//! MOV L, A
//! MOV M, D
//! MOV L, H
//! MOV H, H
//! ```
//!
//! # Examples
//!
//! To disassemble a file to a specific output file
//!
//! ```sh
//! $ i8080 dis rsc/bin/simple.bin
//! MVI A, 0xde
//! MVI B, 0xad
//! ADD B
//! HLT
//! ```

use crate::meta::I8080_OP_META;
use crate::util::vec_u8_to_u16;

use super::errors::DisassembleError;

/// Disassemble the instruction at `from`, returning its text and width in bytes
pub fn disassemble_instruction(v: &[u8], from: usize) -> Result<(String, usize), DisassembleError> {
    let v = v.get(from..).unwrap_or_default();
    let inst = *v.first().ok_or(DisassembleError::NoRemainingBytes(from))?;
    let meta = I8080_OP_META[usize::from(inst)];
    if v.len() < meta.width() {
        return Err(DisassembleError::NotEnoughBytes(from, meta.op, v.to_vec()));
    }
    let mut op: String = meta.op.to_owned();
    if meta.argb || meta.argw {
        if meta.asm_arg_count == 2 {
            op.push(',');
        }
        if meta.argb {
            op.push_str(&format!(" {:#04x}", v[1]));
        } else {
            op.push_str(&format!(" {:#06x}", vec_u8_to_u16(&v[1..3])));
        }
    }
    Ok((op, meta.width()))
}

pub fn disassemble_vec(v: &[u8]) -> Result<Vec<String>, DisassembleError> {
    let mut strings = Vec::new();
    let mut from = 0;
    while from < v.len() {
        let (s, width) = disassemble_instruction(v, from)?;
        strings.push(s);
        from += width;
    }
    Ok(strings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructions_with_arguments() {
        let v = [0x3e, 0xde, 0x01, 0x34, 0x12, 0xc3, 0xef, 0xbe];
        assert_eq!(
            disassemble_vec(&v).unwrap(),
            vec!["MVI A, 0xde", "LXI B, 0x1234", "JMP 0xbeef"]
        );
    }

    #[test]
    fn truncated_input() {
        assert!(matches!(
            disassemble_vec(&[0xc3, 0x00]),
            Err(DisassembleError::NotEnoughBytes(0, "JMP", _))
        ));
        assert!(matches!(
            disassemble_instruction(&[0x00], 5),
            Err(DisassembleError::NoRemainingBytes(5))
        ));
    }
}
