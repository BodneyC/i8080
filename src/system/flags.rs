use crate::util;

#[derive(Debug, Default)]
pub struct Flags {
    pub sign: bool,
    pub zero: bool,
    pub aux_carry: bool,
    pub parity: bool,
    pub carry: bool,
}

impl Flags {
    pub fn new() -> Self {
        Default::default()
    }

    // pub fn debug_line(&self) -> String {
    //     format!(
    //         "S[{}] Z[{}] AC[{}] P[{}] C[{}]",
    //         self.sign as u8,
    //         self.zero as u8,
    //         self.aux_carry as u8,
    //         self.parity as u8,
    //         self.carry as u8,
    //     )
    // }

    pub fn to_byte(&self) -> u8 {
        (self.sign as u8) << 7
            | (self.zero as u8) << 6
            | (self.aux_carry as u8) << 4
            | (self.parity as u8) << 2
            | 0x10 // Always set
            | (self.carry as u8)
    }

    pub fn from_byte(&mut self, byte: u8) {
        self.sign = util::is_bit_set(byte, 7);
        self.zero = util::is_bit_set(byte, 6);
        self.aux_carry = util::is_bit_set(byte, 4);
        self.parity = util::is_bit_set(byte, 2);
        self.carry = util::is_bit_set(byte, 0);
    }

    fn parity(byte: u8) -> bool {
        let mut ones = 0;
        for i in 0..8 {
            ones += util::is_bit_set(byte, i) as u32;
        }
        (ones & 1) == 0
    }

    pub fn zero_sign_parity(&mut self, byte: u8) {
        self.zero = byte == 0;
        self.sign = byte >> 7 == 1;
        self.parity = Flags::parity(byte);
    }
}
