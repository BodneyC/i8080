#[derive(Copy, Clone, Debug)]
pub struct InstMeta {
    pub op: &'static str,
    pub argb: bool,
    pub argw: bool,
    // The cycle counts used are from https://github.com/superzazu/8080/blob/master/i8080.c#L7
    pub cycles: u8,
}

impl Default for InstMeta {
    fn default() -> Self {
        Self {
            op: "",
            argb: false,
            argw: false,
            cycles: 0,
        }
    }
}

impl InstMeta {
    fn new_no_args(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            ..Default::default()
        }
    }

    fn new_argb(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            argb: true,
            ..Default::default()
        }
    }

    fn new_argw(op: &'static str, cycles: u8) -> Self {
        Self {
            op,
            cycles,
            argw: true,
            ..Default::default()
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

pub fn i8080_instruction_meta() -> [InstMeta; 0x100] {
    let mut set: [InstMeta; 0x100] = [Default::default(); 0x100];

    // ------------------------------------------ MOV

    set[0x40] = InstMeta::new_no_args("MOV B B", 5);
    set[0x41] = InstMeta::new_no_args("MOV B C", 5);
    set[0x42] = InstMeta::new_no_args("MOV B D", 5);
    set[0x43] = InstMeta::new_no_args("MOV B E", 5);
    set[0x44] = InstMeta::new_no_args("MOV B H", 5);
    set[0x45] = InstMeta::new_no_args("MOV B L", 5);
    set[0x46] = InstMeta::new_no_args("MOV B M", 7);
    set[0x47] = InstMeta::new_no_args("MOV B A", 5);

    set[0x48] = InstMeta::new_no_args("MOV C B", 5);
    set[0x49] = InstMeta::new_no_args("MOV C C", 5);
    set[0x4a] = InstMeta::new_no_args("MOV C D", 5);
    set[0x4b] = InstMeta::new_no_args("MOV C E", 5);
    set[0x4c] = InstMeta::new_no_args("MOV C H", 5);
    set[0x4d] = InstMeta::new_no_args("MOV C L", 5);
    set[0x4e] = InstMeta::new_no_args("MOV C M", 7);
    set[0x4f] = InstMeta::new_no_args("MOV C A", 5);

    set[0x50] = InstMeta::new_no_args("MOV D B", 5);
    set[0x51] = InstMeta::new_no_args("MOV D C", 5);
    set[0x52] = InstMeta::new_no_args("MOV D D", 5);
    set[0x53] = InstMeta::new_no_args("MOV D E", 5);
    set[0x54] = InstMeta::new_no_args("MOV D H", 5);
    set[0x55] = InstMeta::new_no_args("MOV D L", 5);
    set[0x56] = InstMeta::new_no_args("MOV D M", 7);
    set[0x57] = InstMeta::new_no_args("MOV D A", 5);

    set[0x58] = InstMeta::new_no_args("MOV E B", 5);
    set[0x59] = InstMeta::new_no_args("MOV E C", 5);
    set[0x5a] = InstMeta::new_no_args("MOV E D", 5);
    set[0x5b] = InstMeta::new_no_args("MOV E E", 5);
    set[0x5c] = InstMeta::new_no_args("MOV E H", 5);
    set[0x5d] = InstMeta::new_no_args("MOV E L", 5);
    set[0x5e] = InstMeta::new_no_args("MOV E M", 7);
    set[0x5f] = InstMeta::new_no_args("MOV E A", 5);

    set[0x60] = InstMeta::new_no_args("MOV H B", 5);
    set[0x61] = InstMeta::new_no_args("MOV H C", 5);
    set[0x62] = InstMeta::new_no_args("MOV H D", 5);
    set[0x63] = InstMeta::new_no_args("MOV H E", 5);
    set[0x64] = InstMeta::new_no_args("MOV H H", 5);
    set[0x65] = InstMeta::new_no_args("MOV H L", 5);
    set[0x66] = InstMeta::new_no_args("MOV H M", 7);
    set[0x67] = InstMeta::new_no_args("MOV H A", 5);

    set[0x68] = InstMeta::new_no_args("MOV L B", 5);
    set[0x69] = InstMeta::new_no_args("MOV L C", 5);
    set[0x6a] = InstMeta::new_no_args("MOV L D", 5);
    set[0x6b] = InstMeta::new_no_args("MOV L E", 5);
    set[0x6c] = InstMeta::new_no_args("MOV L H", 5);
    set[0x6d] = InstMeta::new_no_args("MOV L L", 5);
    set[0x6e] = InstMeta::new_no_args("MOV L M", 7);
    set[0x6f] = InstMeta::new_no_args("MOV L A", 5);

    set[0x70] = InstMeta::new_no_args("MOV M B", 7);
    set[0x71] = InstMeta::new_no_args("MOV M C", 7);
    set[0x72] = InstMeta::new_no_args("MOV M D", 7);
    set[0x73] = InstMeta::new_no_args("MOV M E", 7);
    set[0x74] = InstMeta::new_no_args("MOV M H", 7);
    set[0x75] = InstMeta::new_no_args("MOV M L", 7);
    //
    set[0x77] = InstMeta::new_no_args("MOV M A", 7);

    set[0x78] = InstMeta::new_no_args("MOV A B", 5);
    set[0x79] = InstMeta::new_no_args("MOV A C", 5);
    set[0x7a] = InstMeta::new_no_args("MOV A D", 5);
    set[0x7b] = InstMeta::new_no_args("MOV A E", 5);
    set[0x7c] = InstMeta::new_no_args("MOV A H", 5);
    set[0x7d] = InstMeta::new_no_args("MOV A L", 5);
    set[0x7e] = InstMeta::new_no_args("MOV A M", 7);
    set[0x7f] = InstMeta::new_no_args("MOV A A", 5);

    // ------------------------------------------ CONDITIONALS

    // Jxx
    set[0xc3] = InstMeta::new_argw("JMP", 10);
    set[0xc2] = InstMeta::new_argw("JNZ", 10);
    set[0xca] = InstMeta::new_argw("JZ", 10);
    set[0xd2] = InstMeta::new_argw("JNC", 10);
    set[0xda] = InstMeta::new_argw("JC", 10);
    set[0xe2] = InstMeta::new_argw("JPO", 10);
    set[0xea] = InstMeta::new_argw("JPE", 10);
    set[0xf2] = InstMeta::new_argw("JP", 10);
    set[0xfa] = InstMeta::new_argw("JM", 10);
    set[0xe9] = InstMeta::new_no_args("PCHL", 5);

    // Cxx
    set[0xcd] = InstMeta::new_argw("CALL", 17);
    // If the condition is matched, 6 cycles are added
    set[0xc4] = InstMeta::new_argw("CNZ", 11);
    set[0xcc] = InstMeta::new_argw("CZ", 11);
    set[0xd4] = InstMeta::new_argw("CNC", 11);
    set[0xdc] = InstMeta::new_argw("CC", 11);
    set[0xe4] = InstMeta::new_argw("CPO", 11);
    set[0xec] = InstMeta::new_argw("CPE", 11);
    set[0xf4] = InstMeta::new_argw("CP", 11);
    set[0xfc] = InstMeta::new_argw("CM", 11);

    // Rxx
    set[0xc9] = InstMeta::new_no_args("RET", 10);
    // If the condition is matched, 6 cycles are added
    set[0xc0] = InstMeta::new_no_args("RNZ", 5);
    set[0xc8] = InstMeta::new_no_args("RZ", 5);
    set[0xd0] = InstMeta::new_no_args("RNC", 5);
    set[0xd8] = InstMeta::new_no_args("RC", 5);
    set[0xe0] = InstMeta::new_no_args("RPO", 5);
    set[0xe8] = InstMeta::new_no_args("RPE", 5);
    set[0xf0] = InstMeta::new_no_args("RP", 5);
    set[0xf8] = InstMeta::new_no_args("RM", 5);

    // ------------------------------------------ IMMEDIATE

    set[0x06] = InstMeta::new_argb("MVI B", 7);
    set[0x0e] = InstMeta::new_argb("MVI C", 7);
    set[0x16] = InstMeta::new_argb("MVI D", 7);
    set[0x1e] = InstMeta::new_argb("MVI E", 7);
    set[0x26] = InstMeta::new_argb("MVI H", 7);
    set[0x2e] = InstMeta::new_argb("MVI L", 7);
    set[0x36] = InstMeta::new_argb("MVI M", 10);
    set[0x3e] = InstMeta::new_argb("MVI A", 7);

    set[0xc6] = InstMeta::new_argb("ADI", 7);
    set[0xce] = InstMeta::new_argb("ACI", 7);
    set[0xd6] = InstMeta::new_argb("SUI", 7);
    set[0xde] = InstMeta::new_argb("SBI", 7);
    set[0xe6] = InstMeta::new_argb("ANI", 7);
    set[0xee] = InstMeta::new_argb("XRI", 7);
    set[0xf6] = InstMeta::new_argb("ORI", 7);
    set[0xfe] = InstMeta::new_argb("CPI", 7);

    // ------------------------------------------ ACCUMULATOR

    set[0x80] = InstMeta::new_no_args("ADD B", 4);
    set[0x81] = InstMeta::new_no_args("ADD C", 4);
    set[0x82] = InstMeta::new_no_args("ADD D", 4);
    set[0x83] = InstMeta::new_no_args("ADD E", 4);
    set[0x84] = InstMeta::new_no_args("ADD H", 4);
    set[0x85] = InstMeta::new_no_args("ADD L", 4);
    set[0x86] = InstMeta::new_no_args("ADD M", 7);
    set[0x87] = InstMeta::new_no_args("ADD A", 4);

    set[0x88] = InstMeta::new_no_args("ADC B", 4);
    set[0x89] = InstMeta::new_no_args("ADC C", 4);
    set[0x8a] = InstMeta::new_no_args("ADC D", 4);
    set[0x8b] = InstMeta::new_no_args("ADC E", 4);
    set[0x8c] = InstMeta::new_no_args("ADC H", 4);
    set[0x8d] = InstMeta::new_no_args("ADC L", 4);
    set[0x8e] = InstMeta::new_no_args("ADC M", 7);
    set[0x8f] = InstMeta::new_no_args("ADC A", 4);

    set[0x90] = InstMeta::new_no_args("SUB B", 4);
    set[0x91] = InstMeta::new_no_args("SUB C", 4);
    set[0x92] = InstMeta::new_no_args("SUB D", 4);
    set[0x93] = InstMeta::new_no_args("SUB E", 4);
    set[0x94] = InstMeta::new_no_args("SUB H", 4);
    set[0x95] = InstMeta::new_no_args("SUB L", 4);
    set[0x96] = InstMeta::new_no_args("SUB M", 7);
    set[0x97] = InstMeta::new_no_args("SUB A", 4);

    set[0x98] = InstMeta::new_no_args("SBB B", 4);
    set[0x99] = InstMeta::new_no_args("SBB C", 4);
    set[0x9a] = InstMeta::new_no_args("SBB D", 4);
    set[0x9b] = InstMeta::new_no_args("SBB E", 4);
    set[0x9c] = InstMeta::new_no_args("SBB H", 4);
    set[0x9d] = InstMeta::new_no_args("SBB L", 4);
    set[0x9e] = InstMeta::new_no_args("SBB M", 7);
    set[0x9f] = InstMeta::new_no_args("SBB A", 4);

    set[0xa0] = InstMeta::new_no_args("ANA B", 4);
    set[0xa1] = InstMeta::new_no_args("ANA C", 4);
    set[0xa2] = InstMeta::new_no_args("ANA D", 4);
    set[0xa3] = InstMeta::new_no_args("ANA E", 4);
    set[0xa4] = InstMeta::new_no_args("ANA H", 4);
    set[0xa5] = InstMeta::new_no_args("ANA L", 4);
    set[0xa6] = InstMeta::new_no_args("ANA M", 7);
    set[0xa7] = InstMeta::new_no_args("ANA A", 4);

    set[0xa8] = InstMeta::new_no_args("XRA B", 4);
    set[0xa9] = InstMeta::new_no_args("XRA C", 4);
    set[0xaa] = InstMeta::new_no_args("XRA D", 4);
    set[0xab] = InstMeta::new_no_args("XRA E", 4);
    set[0xac] = InstMeta::new_no_args("XRA H", 4);
    set[0xad] = InstMeta::new_no_args("XRA L", 4);
    set[0xae] = InstMeta::new_no_args("XRA M", 7);
    set[0xaf] = InstMeta::new_no_args("XRA A", 4);

    set[0xb0] = InstMeta::new_no_args("ORA B", 4);
    set[0xb1] = InstMeta::new_no_args("ORA C", 4);
    set[0xb2] = InstMeta::new_no_args("ORA D", 4);
    set[0xb3] = InstMeta::new_no_args("ORA E", 4);
    set[0xb4] = InstMeta::new_no_args("ORA H", 4);
    set[0xb5] = InstMeta::new_no_args("ORA L", 4);
    set[0xb6] = InstMeta::new_no_args("ORA M", 7);
    set[0xb7] = InstMeta::new_no_args("ORA A", 4);

    set[0xb8] = InstMeta::new_no_args("CMP B", 4);
    set[0xb9] = InstMeta::new_no_args("CMP C", 4);
    set[0xba] = InstMeta::new_no_args("CMP D", 4);
    set[0xbb] = InstMeta::new_no_args("CMP E", 4);
    set[0xbc] = InstMeta::new_no_args("CMP H", 4);
    set[0xbd] = InstMeta::new_no_args("CMP L", 4);
    set[0xbe] = InstMeta::new_no_args("CMP M", 7);
    set[0xbf] = InstMeta::new_no_args("CMP A", 4);

    // ------------------------------------------ SPECIALS

    set[0x27] = InstMeta::new_no_args("DAA", 4);
    set[0x2f] = InstMeta::new_no_args("CMA", 4);
    set[0x37] = InstMeta::new_no_args("STC", 4);
    set[0x3f] = InstMeta::new_no_args("CMC", 4);
    set[0xeb] = InstMeta::new_no_args("XCHG", 4);

    // ------------------------------------------ UNDOC-NOP

    set[0x08] = InstMeta::new_no_args("---", 4);
    set[0x10] = InstMeta::new_no_args("---", 4);
    set[0x18] = InstMeta::new_no_args("---", 4);
    set[0x20] = InstMeta::new_no_args("---", 4);
    set[0x28] = InstMeta::new_no_args("---", 4);
    set[0x30] = InstMeta::new_no_args("---", 4);
    set[0x38] = InstMeta::new_no_args("---", 4);
    set[0xcb] = InstMeta::new_no_args("---", 4);
    set[0xd9] = InstMeta::new_no_args("---", 4);
    set[0xdd] = InstMeta::new_no_args("---", 4);
    set[0xed] = InstMeta::new_no_args("---", 4);
    set[0xfd] = InstMeta::new_no_args("---", 4);

    // ------------------------------------------ CONTROL

    set[0x00] = InstMeta::new_no_args("NOP", 4);
    set[0x76] = InstMeta::new_no_args("HLT", 7);
    set[0xf3] = InstMeta::new_no_args("DI", 4);
    set[0xfb] = InstMeta::new_no_args("EI", 4);

    // ------------------------------------------ LXI

    set[0x01] = InstMeta::new_argw("LXI B", 4);
    set[0x11] = InstMeta::new_argw("LXI D", 4);
    set[0x21] = InstMeta::new_argw("LXI H", 4);
    set[0x31] = InstMeta::new_argw("LXI SP", 4);

    // ------------------------------------------ LOAD/STORE

    set[0x0a] = InstMeta::new_no_args("LDAX B", 7);
    set[0x1a] = InstMeta::new_no_args("LDAX D", 7);
    set[0x2a] = InstMeta::new_argw("LHDL", 16);
    set[0x3a] = InstMeta::new_argw("LDA", 13);

    set[0x02] = InstMeta::new_no_args("STAX B", 7);
    set[0x12] = InstMeta::new_no_args("STAX D", 7);
    set[0x22] = InstMeta::new_argw("SHDL", 16);
    set[0x32] = InstMeta::new_argw("STA", 13);

    // ------------------------------------------ ROTATE

    set[0x07] = InstMeta::new_no_args("RLD", 4);
    set[0x0f] = InstMeta::new_no_args("RRC", 4);
    set[0x17] = InstMeta::new_no_args("RAL", 4);
    set[0x1f] = InstMeta::new_no_args("RAR", 4);

    // ------------------------------------------ DAD

    set[0x09] = InstMeta::new_argb("DAD B", 10);
    set[0x19] = InstMeta::new_argb("DAD D", 10);
    set[0x29] = InstMeta::new_argb("DAD H", 10);
    set[0x39] = InstMeta::new_argb("DAD SP", 10);

    // ------------------------------------------ INC

    set[0x04] = InstMeta::new_no_args("INR B", 5);
    set[0x0c] = InstMeta::new_no_args("INR C", 5);
    set[0x14] = InstMeta::new_no_args("INR D", 5);
    set[0x1c] = InstMeta::new_no_args("INR E", 5);
    set[0x24] = InstMeta::new_no_args("INR H", 5);
    set[0x2c] = InstMeta::new_no_args("INR L", 5);
    set[0x34] = InstMeta::new_no_args("INR M", 10);
    set[0x3c] = InstMeta::new_no_args("INR A", 5);

    set[0x03] = InstMeta::new_no_args("INX B", 5);
    set[0x13] = InstMeta::new_no_args("INX D", 5);
    set[0x23] = InstMeta::new_no_args("INX H", 5);
    set[0x33] = InstMeta::new_no_args("INX SP", 5);

    // ------------------------------------------ DEC

    set[0x05] = InstMeta::new_no_args("DCR B", 5);
    set[0x0d] = InstMeta::new_no_args("DCR C", 5);
    set[0x15] = InstMeta::new_no_args("DCR D", 5);
    set[0x1d] = InstMeta::new_no_args("DCR E", 5);
    set[0x25] = InstMeta::new_no_args("DCR H", 5);
    set[0x2d] = InstMeta::new_no_args("DCR L", 5);
    set[0x35] = InstMeta::new_no_args("DCR M", 10);
    set[0x3d] = InstMeta::new_no_args("DCR A", 5);

    set[0x0b] = InstMeta::new_no_args("DCX B", 5);
    set[0x1b] = InstMeta::new_no_args("DCX D", 5);
    set[0x2b] = InstMeta::new_no_args("DCX H", 5);
    set[0x3b] = InstMeta::new_no_args("DCX SP", 5);

    // ------------------------------------------ STACK

    set[0xc5] = InstMeta::new_no_args("PUSH B", 11);
    set[0xd5] = InstMeta::new_no_args("PUSH D", 11);
    set[0xe5] = InstMeta::new_no_args("PUSH H", 11);
    set[0xf5] = InstMeta::new_no_args("PUSH PSW", 11);

    set[0xc1] = InstMeta::new_no_args("POP B", 10);
    set[0xd1] = InstMeta::new_no_args("POP D", 10);
    set[0xe1] = InstMeta::new_no_args("POP H", 10);
    set[0xf1] = InstMeta::new_no_args("POP PSW", 10);

    set[0xe3] = InstMeta::new_no_args("XTHL", 18);
    set[0xf9] = InstMeta::new_no_args("SPHL", 5);

    // ------------------------------------------ IO

    set[0xd3] = InstMeta::new_argb("OUT", 10);
    set[0xdb] = InstMeta::new_argb("IN", 10);

    // ------------------------------------------ RESTART

    set[0xc7] = InstMeta::new_no_args("RST 0", 11);
    set[0xcf] = InstMeta::new_no_args("RST 1", 11);
    set[0xd7] = InstMeta::new_no_args("RST 2", 11);
    set[0xdf] = InstMeta::new_no_args("RST 3", 11);
    set[0xe7] = InstMeta::new_no_args("RST 4", 11);
    set[0xef] = InstMeta::new_no_args("RST 5", 11);
    set[0xf7] = InstMeta::new_no_args("RST 6", 11);
    set[0xff] = InstMeta::new_no_args("RST 7", 11);

    set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_full() {
        let inst_meta = i8080_instruction_meta();
        for i in 0x00..=0xff {
            let inst_desc = format!("{:#04x} ({:#03}): {}", i, i, inst_meta[i as usize].op);
            assert!(
                inst_meta[i as usize].op.len() > 0,
                "Name not set: {}",
                inst_desc
            );
            assert!(
                inst_meta[i as usize].cycles != 0,
                "Cycles not set: {}",
                inst_desc
            );
        }
    }
}
