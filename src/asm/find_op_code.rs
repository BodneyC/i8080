use crate::op_meta::I8080_OP_META;

use super::errors::OpParseError;

pub fn from_args(inst: &str, arg0: u16, arg1: u16) -> Result<usize, OpParseError> {
    from_args_and_sp_psw(inst, arg0, arg1, false, false)
}

pub fn from_args_and_sp_psw(
    inst: &str,
    arg0: u16,
    arg1: u16,
    sp: bool,
    psw: bool,
) -> Result<usize, OpParseError> {
    let extra = sp || psw;
    match inst {
        "MOV" => {
            let meta_res = with_all_regs(0x40, arg0, arg1, extra);
            if let Ok(meta) = meta_res {
                if meta == 0x76 {
                    Err(OpParseError::MovAsHalt)
                } else {
                    Ok(meta)
                }
            } else {
                meta_res
            }
        }
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

        "LXI" => with_word_regs(0x01, arg0, sp),
        "DAD" => with_word_regs(0x01, arg0, sp),
        "INX" => with_word_regs(0x01, arg0, sp),
        "DCX" => with_word_regs(0x01, arg0, sp),
        "PUSH" => with_word_regs(0x01, arg0, psw),
        "POP" => with_word_regs(0x01, arg0, psw),

        "LDAX" => with_b_or_d(0x01, arg0, extra),
        "STAX" => with_b_or_d(0x01, arg0, extra),

        _ => I8080_OP_META
            .into_iter()
            .position(|meta| inst == meta.op)
            .ok_or(OpParseError::NoSuchInstruction(inst.to_string())),
    }
}

fn with_all_regs(start: u16, arg0: u16, arg1: u16, extra: bool) -> Result<usize, OpParseError> {
    if extra {
        Err(OpParseError::UnknownRegister)
    } else if arg0 > 7 || arg1 > 7 {
        Err(OpParseError::UnknownRegister)
    } else {
        let idx = start + (arg0 * 8) + arg1;
        Ok(idx as usize)
    }
}

fn with_b_or_d(start: u16, arg0: u16, extra: bool) -> Result<usize, OpParseError> {
    if extra {
        Err(OpParseError::InvalidRegister)
    } else if arg0 == 0 || arg0 == 2 {
        let idx = start + ((arg0 / 2) * 0x10);
        Ok(idx as usize)
    } else {
        Err(OpParseError::InvalidRegister)
    }
}

fn with_word_regs(start: u16, arg0: u16, extra: bool) -> Result<usize, OpParseError> {
    // Accounts for SP or PSW, bit hacky...
    let offset: u16 = if extra {
        3
    } else if arg0 == 0 || arg0 == 2 || arg0 == 4 {
        arg0 / 2
    } else {
        return Err(OpParseError::InvalidRegister);
    };
    let idx = start + (offset as u16 * 0x10);
    Ok(idx as usize)
}
