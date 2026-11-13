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

    /// The flags as the byte pushed by `PUSH PSW`, bit 1 is always set and bits 3 and 5 unset
    pub fn to_byte(&self) -> u8 {
        (self.sign as u8) << 7
            | (self.zero as u8) << 6
            | (self.aux_carry as u8) << 4
            | (self.parity as u8) << 2
            | 0x02
            | (self.carry as u8)
    }

    pub fn set_from_byte(&mut self, byte: u8) {
        self.sign = util::is_bit_set(byte, 7);
        self.zero = util::is_bit_set(byte, 6);
        self.aux_carry = util::is_bit_set(byte, 4);
        self.parity = util::is_bit_set(byte, 2);
        self.carry = util::is_bit_set(byte, 0);
    }

    /// Even parity
    fn parity(byte: u8) -> bool {
        byte.count_ones().is_multiple_of(2)
    }

    pub fn zero_sign_parity(&mut self, byte: u8) {
        self.zero = byte == 0;
        self.sign = byte >> 7 == 1;
        self.parity = Flags::parity(byte);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_round_trip() {
        let mut flags = Flags::new();
        flags.set_from_byte(0xff);
        assert_eq!(flags.to_byte(), 0xd7, "unused bits are fixed");
        flags.set_from_byte(0x00);
        assert_eq!(flags.to_byte(), 0x02, "bit 1 is always set");
        assert!(!flags.aux_carry);
    }
}
