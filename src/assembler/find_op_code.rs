use crate::errors::OpParseError;
use crate::op_meta::I8080_OP_META;

pub fn from_args(inst: String, arg0: u16, arg1: u16) -> Result<usize, OpParseError> {
    from_args_and_sp_psw(inst, arg0, arg1, false, false)
}

pub fn from_args_and_sp_psw(
    inst: String,
    arg0: u16,
    arg1: u16,
    sp: bool,
    psw: bool,
) -> Result<usize, OpParseError> {
    match inst.as_str() {
        "MOV" => {
            let meta_res = with_all_regs(0x40, arg0, arg1);
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
        "MVI" => with_all_regs(0x06, arg0, 0),
        "ADD" => with_all_regs(0x80, 0, arg0),
        "ADC" => with_all_regs(0x88, 0, arg0),
        "SUB" => with_all_regs(0x90, 0, arg0),
        "SBB" => with_all_regs(0x98, 0, arg0),
        "ANA" => with_all_regs(0xa0, 0, arg0),
        "XRA" => with_all_regs(0xa8, 0, arg0),
        "ORA" => with_all_regs(0xb0, 0, arg0),
        "CMP" => with_all_regs(0xb8, 0, arg0),
        "INR" => with_all_regs(0x04, arg0, 0),
        "DCR" => with_all_regs(0x05, arg0, 0),
        "RST" => with_all_regs(0xc7, arg0, 0),

        "LXI" => with_word_regs(0x01, arg0, sp),
        "DAD" => with_word_regs(0x01, arg0, sp),
        "INX" => with_word_regs(0x01, arg0, sp),
        "DCX" => with_word_regs(0x01, arg0, sp),
        "PUSH" => with_word_regs(0x01, arg0, psw),
        "POP" => with_word_regs(0x01, arg0, psw),

        "LDAX" => with_b_or_d(0x01, arg0),
        "STAX" => with_b_or_d(0x01, arg0),

        _ => I8080_OP_META
            .into_iter()
            .position(|meta| inst.as_str() == meta.op)
            .ok_or(OpParseError::NoSuchInstruction(inst)),
    }
}

fn with_all_regs(start: u16, arg0: u16, arg1: u16) -> Result<usize, OpParseError> {
    if arg0 > 7 || arg1 > 7 {
        Err(OpParseError::InvalidRegister)
    } else {
        let idx = start + (arg0 * 8) + arg1;
        Ok(idx as usize)
    }
}

fn with_b_or_d(start: u16, arg0: u16) -> Result<usize, OpParseError> {
    if arg0 == 0 || arg0 == 2 {
        let idx = start + ((arg0 / 2) * 0x10);
        Ok(idx as usize)
    } else {
        Err(OpParseError::InvalidRegister)
    }
}

fn with_word_regs(start: u16, arg0: u16, extra_valid: bool) -> Result<usize, OpParseError> {
    // Accounts for SP or PSW, bit hacky...
    let offset: u16 = if extra_valid {
        3
    } else if arg0 == 0 || arg0 == 2 || arg0 == 4 {
        arg0 / 2
    } else {
        return Err(OpParseError::InvalidRegister);
    };
    let idx = start + (offset as u16 * 0x10);
    Ok(idx as usize)
}
