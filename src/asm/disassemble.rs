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

use crate::meta::I8080_OP_META;
use crate::util::vec_u8_to_u16;

use super::errors::DisassembleError;

pub fn disassemble_instruction(v: &[u8], from: usize) -> Result<(String, usize), DisassembleError> {
    let v = &v[from..];
    let inst = v.get(0);
    if inst.is_none() {
        return Err(DisassembleError::NoRemainingBytes(from));
    }
    let inst = inst.unwrap();
    let meta = I8080_OP_META[*inst as usize];
    if v.len() < meta.width() {
        return Err(DisassembleError::NotEnoughBytes(from, meta.op, v.to_vec()));
    }
    let mut op: String = meta.op.to_owned();
    if meta.argb {
        if meta.asm_arg_count == 2 {
            op.push_str(",");
        }
        op.push_str(&format!(" {:#04x}", v.get(1).unwrap()));
    } else if meta.argw {
        if meta.asm_arg_count == 2 {
            op.push_str(",");
        }
        op.push_str(&format!(" {:#06x}", vec_u8_to_u16(v)));
    }
    Ok((op, meta.width()))
}

pub fn disassemble_vec(v: &Vec<u8>) -> Result<Vec<String>, DisassembleError> {
    let mut strings = Vec::new();
    let mut from = 0;
    while from < v.len() {
        let (s, width) = disassemble_instruction(&v, from)?;
        strings.push(s);
        // Shouldn't happen but peace of mind is nice
        if width == 0 {
            let to = if from + 3 > v.len() {
                from + 3
            } else {
                v.len()
            };
            return Err(DisassembleError::UnknownError(from, v[from..to].to_vec()));
        }
        from += width;
    }
    Ok(strings)
}
