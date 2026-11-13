//! Find the operation code for a given string
//!
//! Depending on what part of the assembly process we're in we may not be able to resolve the true
//! operation code.
//!
//! Say we have `MOV A, _label`, without knowing what that `_label`'s value is, we cannot know the
//! true op-code.
//!
//! We can get some information, all `MOV X, Y` instructions have the same metadata, so depending
//! on the arguments provided to these methods, you will either get an op-code estimation or the
//! real thing.
//!
//! Then, when labels have been resolved, we can use the same method to find the real code.

use crate::meta::I8080_OP_META;

use super::errors::OpParseError;

/// Estimate the op-code of an instruction before its arguments are known
pub fn from_args(inst: &str, arg0: u16, arg1: u16) -> Result<usize, OpParseError> {
    from_args_and_sp_psw(inst, arg0, arg1, false, false)
}

/// Find the op-code of an instruction from its resolved arguments
///
/// `sp` and `psw` indicate the first argument was the `SP` or `PSW` register respectively
pub fn from_args_and_sp_psw(
    inst: &str,
    arg0: u16,
    arg1: u16,
    sp: bool,
    psw: bool,
) -> Result<usize, OpParseError> {
    let extra = sp || psw;
    match inst {
        "MOV" => match with_all_regs(0x40, arg0, arg1, extra)? {
            0x76 => Err(OpParseError::MovAsHalt),
            code => Ok(code),
        },
        "MVI" => with_all_regs(0x06, arg0, 0, extra),
        "ADD" => with_all_regs(0x80, 0, arg0, extra),
        "ADC" => with_all_regs(0x88, 0, arg0, extra),
        "SUB" => with_all_regs(0x90, 0, arg0, extra),
        "SBB" => with_all_regs(0x98, 0, arg0, extra),
        "ANA" => with_all_regs(0xa0, 0, arg0, extra),
        "XRA" => with_all_regs(0xa8, 0, arg0, extra),
        "ORA" => with_all_regs(0xb0, 0, arg0, extra),
        "CMP" => with_all_regs(0xb8, 0, arg0, extra),
        "INR" => with_all_regs(0x04, arg0, 0, extra),
        "DCR" => with_all_regs(0x05, arg0, 0, extra),
        "RST" => with_all_regs(0xc7, arg0, 0, extra),

        "LXI" | "DAD" | "INX" | "DCX" if psw => Err(OpParseError::InvalidRegister),
        "LXI" => with_word_regs(0x01, arg0, sp),
        "DAD" => with_word_regs(0x09, arg0, sp),
        "INX" => with_word_regs(0x03, arg0, sp),
        "DCX" => with_word_regs(0x0b, arg0, sp),

        "PUSH" | "POP" if sp => Err(OpParseError::InvalidRegister),
        "PUSH" => with_word_regs(0xc5, arg0, psw),
        "POP" => with_word_regs(0xc1, arg0, psw),

        "LDAX" => with_b_or_d(0x0a, arg0, extra),
        "STAX" => with_b_or_d(0x02, arg0, extra),

        _ => I8080_OP_META
            .into_iter()
            .position(|meta| inst == meta.op)
            .ok_or_else(|| OpParseError::NoSuchInstruction(inst.to_string())),
    }
}

fn with_all_regs(start: u16, arg0: u16, arg1: u16, extra: bool) -> Result<usize, OpParseError> {
    if extra || arg0 > 7 || arg1 > 7 {
        Err(OpParseError::UnknownRegister)
    } else {
        let idx = start + (arg0 * 8) + arg1;
        Ok(idx as usize)
    }
}

/// Register pair B (0) or D (2)
fn with_b_or_d(start: u16, arg0: u16, extra: bool) -> Result<usize, OpParseError> {
    match arg0 {
        0 | 2 if !extra => Ok((start + (arg0 / 2) * 0x10) as usize),
        _ => Err(OpParseError::InvalidRegister),
    }
}

/// Register pair B (0), D (2), H (4), or SP/PSW (`extra`)
fn with_word_regs(start: u16, arg0: u16, extra: bool) -> Result<usize, OpParseError> {
    let offset = match arg0 {
        _ if extra => 3,
        0 | 2 | 4 => arg0 / 2,
        _ => return Err(OpParseError::InvalidRegister),
    };
    Ok((start + offset * 0x10) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_pair_instructions() {
        assert_eq!(from_args("LDAX", 0, 0).unwrap(), 0x0a);
        assert_eq!(from_args("LDAX", 2, 0).unwrap(), 0x1a);
        assert_eq!(from_args("STAX", 0, 0).unwrap(), 0x02);
        assert_eq!(from_args("STAX", 2, 0).unwrap(), 0x12);
        assert!(from_args("LDAX", 4, 0).is_err());
        assert_eq!(
            from_args_and_sp_psw("LXI", 0, 0, true, false).unwrap(),
            0x31
        );
        assert_eq!(
            from_args_and_sp_psw("PUSH", 0, 0, false, true).unwrap(),
            0xf5
        );
        assert!(from_args_and_sp_psw("LXI", 0, 0, false, true).is_err());
        assert!(from_args_and_sp_psw("PUSH", 0, 0, true, false).is_err());
    }

    #[test]
    fn mov_m_m_is_not_halt() {
        assert!(matches!(
            from_args("MOV", 6, 6),
            Err(OpParseError::MovAsHalt)
        ));
    }

    #[test]
    fn named_instructions() {
        assert_eq!(from_args("RLC", 0, 0).unwrap(), 0x07);
        assert_eq!(from_args("SHLD", 0, 0).unwrap(), 0x22);
        assert_eq!(from_args("LHLD", 0, 0).unwrap(), 0x2a);
        assert!(matches!(
            from_args("NOPE", 0, 0),
            Err(OpParseError::NoSuchInstruction(_))
        ));
    }
}
