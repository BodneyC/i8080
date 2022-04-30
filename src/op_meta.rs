#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OpMeta {
    pub op: &'static str,
    pub argb: bool,
    pub argw: bool,
    // The cycle counts used are from https://github.com/superzazu/8080/blob/master/i8080.c#L7
    pub cycles: u8,
}

impl Default for OpMeta {
    fn default() -> Self {
        Self {
            op: "",
            argb: false,
            argw: false,
            cycles: 0,
        }
    }
}

impl OpMeta {
    const fn new_no_args(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            argb: false,
            argw: false,
        }
    }

    const fn new_argb(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            argb: true,
            argw: false,
        }
    }

    const fn new_argw(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            argb: false,
            argw: true,
        }
    }

    pub fn width(&self) -> usize {
        if self.argw {
            3
        } else if self.argb {
            2
        } else {
            1
        }
    }
}

pub const I8080_OP_META: [OpMeta; 0x100] = load_op_meta();

const fn load_op_meta() -> [OpMeta; 0x100] {
    let mut set: [OpMeta; 0x100] = [OpMeta::new_no_args("", 0); 0x100];

    // ------------------------------------------ MOV

    set[0x40] = OpMeta::new_no_args("MOV B B", 5);
    set[0x41] = OpMeta::new_no_args("MOV B C", 5);
    set[0x42] = OpMeta::new_no_args("MOV B D", 5);
    set[0x43] = OpMeta::new_no_args("MOV B E", 5);
    set[0x44] = OpMeta::new_no_args("MOV B H", 5);
    set[0x45] = OpMeta::new_no_args("MOV B L", 5);
    set[0x46] = OpMeta::new_no_args("MOV B M", 7);
    set[0x47] = OpMeta::new_no_args("MOV B A", 5);

    set[0x48] = OpMeta::new_no_args("MOV C B", 5);
    set[0x49] = OpMeta::new_no_args("MOV C C", 5);
    set[0x4a] = OpMeta::new_no_args("MOV C D", 5);
    set[0x4b] = OpMeta::new_no_args("MOV C E", 5);
    set[0x4c] = OpMeta::new_no_args("MOV C H", 5);
    set[0x4d] = OpMeta::new_no_args("MOV C L", 5);
    set[0x4e] = OpMeta::new_no_args("MOV C M", 7);
    set[0x4f] = OpMeta::new_no_args("MOV C A", 5);

    set[0x50] = OpMeta::new_no_args("MOV D B", 5);
    set[0x51] = OpMeta::new_no_args("MOV D C", 5);
    set[0x52] = OpMeta::new_no_args("MOV D D", 5);
    set[0x53] = OpMeta::new_no_args("MOV D E", 5);
    set[0x54] = OpMeta::new_no_args("MOV D H", 5);
    set[0x55] = OpMeta::new_no_args("MOV D L", 5);
    set[0x56] = OpMeta::new_no_args("MOV D M", 7);
    set[0x57] = OpMeta::new_no_args("MOV D A", 5);

    set[0x58] = OpMeta::new_no_args("MOV E B", 5);
    set[0x59] = OpMeta::new_no_args("MOV E C", 5);
    set[0x5a] = OpMeta::new_no_args("MOV E D", 5);
    set[0x5b] = OpMeta::new_no_args("MOV E E", 5);
    set[0x5c] = OpMeta::new_no_args("MOV E H", 5);
    set[0x5d] = OpMeta::new_no_args("MOV E L", 5);
    set[0x5e] = OpMeta::new_no_args("MOV E M", 7);
    set[0x5f] = OpMeta::new_no_args("MOV E A", 5);

    set[0x60] = OpMeta::new_no_args("MOV H B", 5);
    set[0x61] = OpMeta::new_no_args("MOV H C", 5);
    set[0x62] = OpMeta::new_no_args("MOV H D", 5);
    set[0x63] = OpMeta::new_no_args("MOV H E", 5);
    set[0x64] = OpMeta::new_no_args("MOV H H", 5);
    set[0x65] = OpMeta::new_no_args("MOV H L", 5);
    set[0x66] = OpMeta::new_no_args("MOV H M", 7);
    set[0x67] = OpMeta::new_no_args("MOV H A", 5);

    set[0x68] = OpMeta::new_no_args("MOV L B", 5);
    set[0x69] = OpMeta::new_no_args("MOV L C", 5);
    set[0x6a] = OpMeta::new_no_args("MOV L D", 5);
    set[0x6b] = OpMeta::new_no_args("MOV L E", 5);
    set[0x6c] = OpMeta::new_no_args("MOV L H", 5);
    set[0x6d] = OpMeta::new_no_args("MOV L L", 5);
    set[0x6e] = OpMeta::new_no_args("MOV L M", 7);
    set[0x6f] = OpMeta::new_no_args("MOV L A", 5);

    set[0x70] = OpMeta::new_no_args("MOV M B", 7);
    set[0x71] = OpMeta::new_no_args("MOV M C", 7);
    set[0x72] = OpMeta::new_no_args("MOV M D", 7);
    set[0x73] = OpMeta::new_no_args("MOV M E", 7);
    set[0x74] = OpMeta::new_no_args("MOV M H", 7);
    set[0x75] = OpMeta::new_no_args("MOV M L", 7);
    //
    set[0x77] = OpMeta::new_no_args("MOV M A", 7);

    set[0x78] = OpMeta::new_no_args("MOV A B", 5);
    set[0x79] = OpMeta::new_no_args("MOV A C", 5);
    set[0x7a] = OpMeta::new_no_args("MOV A D", 5);
    set[0x7b] = OpMeta::new_no_args("MOV A E", 5);
    set[0x7c] = OpMeta::new_no_args("MOV A H", 5);
    set[0x7d] = OpMeta::new_no_args("MOV A L", 5);
    set[0x7e] = OpMeta::new_no_args("MOV A M", 7);
    set[0x7f] = OpMeta::new_no_args("MOV A A", 5);

    // ------------------------------------------ CONDITIONALS

    // Jxx
    set[0xc3] = OpMeta::new_argw("JMP", 10);
    set[0xc2] = OpMeta::new_argw("JNZ", 10);
    set[0xca] = OpMeta::new_argw("JZ", 10);
    set[0xd2] = OpMeta::new_argw("JNC", 10);
    set[0xda] = OpMeta::new_argw("JC", 10);
    set[0xe2] = OpMeta::new_argw("JPO", 10);
    set[0xea] = OpMeta::new_argw("JPE", 10);
    set[0xf2] = OpMeta::new_argw("JP", 10);
    set[0xfa] = OpMeta::new_argw("JM", 10);
    set[0xe9] = OpMeta::new_no_args("PCHL", 5);

    // Cxx
    set[0xcd] = OpMeta::new_argw("CALL", 17);
    // If the condition is matched, 6 cycles are added
    set[0xc4] = OpMeta::new_argw("CNZ", 11);
    set[0xcc] = OpMeta::new_argw("CZ", 11);
    set[0xd4] = OpMeta::new_argw("CNC", 11);
    set[0xdc] = OpMeta::new_argw("CC", 11);
    set[0xe4] = OpMeta::new_argw("CPO", 11);
    set[0xec] = OpMeta::new_argw("CPE", 11);
    set[0xf4] = OpMeta::new_argw("CP", 11);
    set[0xfc] = OpMeta::new_argw("CM", 11);

    // Rxx
    set[0xc9] = OpMeta::new_no_args("RET", 10);
    // If the condition is matched, 6 cycles are added
    set[0xc0] = OpMeta::new_no_args("RNZ", 5);
    set[0xc8] = OpMeta::new_no_args("RZ", 5);
    set[0xd0] = OpMeta::new_no_args("RNC", 5);
    set[0xd8] = OpMeta::new_no_args("RC", 5);
    set[0xe0] = OpMeta::new_no_args("RPO", 5);
    set[0xe8] = OpMeta::new_no_args("RPE", 5);
    set[0xf0] = OpMeta::new_no_args("RP", 5);
    set[0xf8] = OpMeta::new_no_args("RM", 5);

    // ------------------------------------------ IMMEDIATE

    set[0x06] = OpMeta::new_argb("MVI B", 7);
    set[0x0e] = OpMeta::new_argb("MVI C", 7);
    set[0x16] = OpMeta::new_argb("MVI D", 7);
    set[0x1e] = OpMeta::new_argb("MVI E", 7);
    set[0x26] = OpMeta::new_argb("MVI H", 7);
    set[0x2e] = OpMeta::new_argb("MVI L", 7);
    set[0x36] = OpMeta::new_argb("MVI M", 10);
    set[0x3e] = OpMeta::new_argb("MVI A", 7);

    set[0xc6] = OpMeta::new_argb("ADI", 7);
    set[0xce] = OpMeta::new_argb("ACI", 7);
    set[0xd6] = OpMeta::new_argb("SUI", 7);
    set[0xde] = OpMeta::new_argb("SBI", 7);
    set[0xe6] = OpMeta::new_argb("ANI", 7);
    set[0xee] = OpMeta::new_argb("XRI", 7);
    set[0xf6] = OpMeta::new_argb("ORI", 7);
    set[0xfe] = OpMeta::new_argb("CPI", 7);

    // ------------------------------------------ ACCUMULATOR

    set[0x80] = OpMeta::new_no_args("ADD B", 4);
    set[0x81] = OpMeta::new_no_args("ADD C", 4);
    set[0x82] = OpMeta::new_no_args("ADD D", 4);
    set[0x83] = OpMeta::new_no_args("ADD E", 4);
    set[0x84] = OpMeta::new_no_args("ADD H", 4);
    set[0x85] = OpMeta::new_no_args("ADD L", 4);
    set[0x86] = OpMeta::new_no_args("ADD M", 7);
    set[0x87] = OpMeta::new_no_args("ADD A", 4);

    set[0x88] = OpMeta::new_no_args("ADC B", 4);
    set[0x89] = OpMeta::new_no_args("ADC C", 4);
    set[0x8a] = OpMeta::new_no_args("ADC D", 4);
    set[0x8b] = OpMeta::new_no_args("ADC E", 4);
    set[0x8c] = OpMeta::new_no_args("ADC H", 4);
    set[0x8d] = OpMeta::new_no_args("ADC L", 4);
    set[0x8e] = OpMeta::new_no_args("ADC M", 7);
    set[0x8f] = OpMeta::new_no_args("ADC A", 4);

    set[0x90] = OpMeta::new_no_args("SUB B", 4);
    set[0x91] = OpMeta::new_no_args("SUB C", 4);
    set[0x92] = OpMeta::new_no_args("SUB D", 4);
    set[0x93] = OpMeta::new_no_args("SUB E", 4);
    set[0x94] = OpMeta::new_no_args("SUB H", 4);
    set[0x95] = OpMeta::new_no_args("SUB L", 4);
    set[0x96] = OpMeta::new_no_args("SUB M", 7);
    set[0x97] = OpMeta::new_no_args("SUB A", 4);

    set[0x98] = OpMeta::new_no_args("SBB B", 4);
    set[0x99] = OpMeta::new_no_args("SBB C", 4);
    set[0x9a] = OpMeta::new_no_args("SBB D", 4);
    set[0x9b] = OpMeta::new_no_args("SBB E", 4);
    set[0x9c] = OpMeta::new_no_args("SBB H", 4);
    set[0x9d] = OpMeta::new_no_args("SBB L", 4);
    set[0x9e] = OpMeta::new_no_args("SBB M", 7);
    set[0x9f] = OpMeta::new_no_args("SBB A", 4);

    set[0xa0] = OpMeta::new_no_args("ANA B", 4);
    set[0xa1] = OpMeta::new_no_args("ANA C", 4);
    set[0xa2] = OpMeta::new_no_args("ANA D", 4);
    set[0xa3] = OpMeta::new_no_args("ANA E", 4);
    set[0xa4] = OpMeta::new_no_args("ANA H", 4);
    set[0xa5] = OpMeta::new_no_args("ANA L", 4);
    set[0xa6] = OpMeta::new_no_args("ANA M", 7);
    set[0xa7] = OpMeta::new_no_args("ANA A", 4);

    set[0xa8] = OpMeta::new_no_args("XRA B", 4);
    set[0xa9] = OpMeta::new_no_args("XRA C", 4);
    set[0xaa] = OpMeta::new_no_args("XRA D", 4);
    set[0xab] = OpMeta::new_no_args("XRA E", 4);
    set[0xac] = OpMeta::new_no_args("XRA H", 4);
    set[0xad] = OpMeta::new_no_args("XRA L", 4);
    set[0xae] = OpMeta::new_no_args("XRA M", 7);
    set[0xaf] = OpMeta::new_no_args("XRA A", 4);

    set[0xb0] = OpMeta::new_no_args("ORA B", 4);
    set[0xb1] = OpMeta::new_no_args("ORA C", 4);
    set[0xb2] = OpMeta::new_no_args("ORA D", 4);
    set[0xb3] = OpMeta::new_no_args("ORA E", 4);
    set[0xb4] = OpMeta::new_no_args("ORA H", 4);
    set[0xb5] = OpMeta::new_no_args("ORA L", 4);
    set[0xb6] = OpMeta::new_no_args("ORA M", 7);
    set[0xb7] = OpMeta::new_no_args("ORA A", 4);

    set[0xb8] = OpMeta::new_no_args("CMP B", 4);
    set[0xb9] = OpMeta::new_no_args("CMP C", 4);
    set[0xba] = OpMeta::new_no_args("CMP D", 4);
    set[0xbb] = OpMeta::new_no_args("CMP E", 4);
    set[0xbc] = OpMeta::new_no_args("CMP H", 4);
    set[0xbd] = OpMeta::new_no_args("CMP L", 4);
    set[0xbe] = OpMeta::new_no_args("CMP M", 7);
    set[0xbf] = OpMeta::new_no_args("CMP A", 4);

    // ------------------------------------------ SPECIALS

    set[0x27] = OpMeta::new_no_args("DAA", 4);
    set[0x2f] = OpMeta::new_no_args("CMA", 4);
    set[0x37] = OpMeta::new_no_args("STC", 4);
    set[0x3f] = OpMeta::new_no_args("CMC", 4);
    set[0xeb] = OpMeta::new_no_args("XCHG", 4);

    // ------------------------------------------ UNDOC-NOP

    set[0x08] = OpMeta::new_no_args("---", 4);
    set[0x10] = OpMeta::new_no_args("---", 4);
    set[0x18] = OpMeta::new_no_args("---", 4);
    set[0x20] = OpMeta::new_no_args("---", 4);
    set[0x28] = OpMeta::new_no_args("---", 4);
    set[0x30] = OpMeta::new_no_args("---", 4);
    set[0x38] = OpMeta::new_no_args("---", 4);
    set[0xcb] = OpMeta::new_no_args("---", 4);
    set[0xd9] = OpMeta::new_no_args("---", 4);
    set[0xdd] = OpMeta::new_no_args("---", 4);
    set[0xed] = OpMeta::new_no_args("---", 4);
    set[0xfd] = OpMeta::new_no_args("---", 4);

    // ------------------------------------------ CONTROL

    set[0x00] = OpMeta::new_no_args("NOP", 4);
    set[0x76] = OpMeta::new_no_args("HLT", 7);
    set[0xf3] = OpMeta::new_no_args("DI", 4);
    set[0xfb] = OpMeta::new_no_args("EI", 4);

    // ------------------------------------------ LXI

    set[0x01] = OpMeta::new_argw("LXI B", 4);
    set[0x11] = OpMeta::new_argw("LXI D", 4);
    set[0x21] = OpMeta::new_argw("LXI H", 4);
    set[0x31] = OpMeta::new_argw("LXI SP", 4);

    // ------------------------------------------ LOAD/STORE

    set[0x0a] = OpMeta::new_no_args("LDAX B", 7);
    set[0x1a] = OpMeta::new_no_args("LDAX D", 7);
    set[0x2a] = OpMeta::new_argw("LHDL", 16);
    set[0x3a] = OpMeta::new_argw("LDA", 13);

    set[0x02] = OpMeta::new_no_args("STAX B", 7);
    set[0x12] = OpMeta::new_no_args("STAX D", 7);
    set[0x22] = OpMeta::new_argw("SHDL", 16);
    set[0x32] = OpMeta::new_argw("STA", 13);

    // ------------------------------------------ ROTATE

    set[0x07] = OpMeta::new_no_args("RLD", 4);
    set[0x0f] = OpMeta::new_no_args("RRC", 4);
    set[0x17] = OpMeta::new_no_args("RAL", 4);
    set[0x1f] = OpMeta::new_no_args("RAR", 4);

    // ------------------------------------------ DAD

    set[0x09] = OpMeta::new_argb("DAD B", 10);
    set[0x19] = OpMeta::new_argb("DAD D", 10);
    set[0x29] = OpMeta::new_argb("DAD H", 10);
    set[0x39] = OpMeta::new_argb("DAD SP", 10);

    // ------------------------------------------ INC

    set[0x04] = OpMeta::new_no_args("INR B", 5);
    set[0x0c] = OpMeta::new_no_args("INR C", 5);
    set[0x14] = OpMeta::new_no_args("INR D", 5);
    set[0x1c] = OpMeta::new_no_args("INR E", 5);
    set[0x24] = OpMeta::new_no_args("INR H", 5);
    set[0x2c] = OpMeta::new_no_args("INR L", 5);
    set[0x34] = OpMeta::new_no_args("INR M", 10);
    set[0x3c] = OpMeta::new_no_args("INR A", 5);

    set[0x03] = OpMeta::new_no_args("INX B", 5);
    set[0x13] = OpMeta::new_no_args("INX D", 5);
    set[0x23] = OpMeta::new_no_args("INX H", 5);
    set[0x33] = OpMeta::new_no_args("INX SP", 5);

    // ------------------------------------------ DEC

    set[0x05] = OpMeta::new_no_args("DCR B", 5);
    set[0x0d] = OpMeta::new_no_args("DCR C", 5);
    set[0x15] = OpMeta::new_no_args("DCR D", 5);
    set[0x1d] = OpMeta::new_no_args("DCR E", 5);
    set[0x25] = OpMeta::new_no_args("DCR H", 5);
    set[0x2d] = OpMeta::new_no_args("DCR L", 5);
    set[0x35] = OpMeta::new_no_args("DCR M", 10);
    set[0x3d] = OpMeta::new_no_args("DCR A", 5);

    set[0x0b] = OpMeta::new_no_args("DCX B", 5);
    set[0x1b] = OpMeta::new_no_args("DCX D", 5);
    set[0x2b] = OpMeta::new_no_args("DCX H", 5);
    set[0x3b] = OpMeta::new_no_args("DCX SP", 5);

    // ------------------------------------------ STACK

    set[0xc5] = OpMeta::new_no_args("PUSH B", 11);
    set[0xd5] = OpMeta::new_no_args("PUSH D", 11);
    set[0xe5] = OpMeta::new_no_args("PUSH H", 11);
    set[0xf5] = OpMeta::new_no_args("PUSH PSW", 11);

    set[0xc1] = OpMeta::new_no_args("POP B", 10);
    set[0xd1] = OpMeta::new_no_args("POP D", 10);
    set[0xe1] = OpMeta::new_no_args("POP H", 10);
    set[0xf1] = OpMeta::new_no_args("POP PSW", 10);

    set[0xe3] = OpMeta::new_no_args("XTHL", 18);
    set[0xf9] = OpMeta::new_no_args("SPHL", 5);

    // ------------------------------------------ IO

    set[0xd3] = OpMeta::new_argb("OUT", 10);
    set[0xdb] = OpMeta::new_argb("IN", 10);

    // ------------------------------------------ RESTART

    set[0xc7] = OpMeta::new_no_args("RST 0", 11);
    set[0xcf] = OpMeta::new_no_args("RST 1", 11);
    set[0xd7] = OpMeta::new_no_args("RST 2", 11);
    set[0xdf] = OpMeta::new_no_args("RST 3", 11);
    set[0xe7] = OpMeta::new_no_args("RST 4", 11);
    set[0xef] = OpMeta::new_no_args("RST 5", 11);
    set[0xf7] = OpMeta::new_no_args("RST 6", 11);
    set[0xff] = OpMeta::new_no_args("RST 7", 11);

    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_full() {
        for i in 0x00..=0xff {
            let inst_desc = format!("{:#04x} ({:#03}): {}", i, i, I8080_OP_META[i as usize].op);
            assert!(
                I8080_OP_META[i as usize].op.len() > 0,
                "Name not set: {}",
                inst_desc
            );
            assert!(
                I8080_OP_META[i as usize].cycles != 0,
                "Cycles not set: {}",
                inst_desc
            );
        }
    }
}
