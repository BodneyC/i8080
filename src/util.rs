pub fn is_bit_set(byte: u8, bit: u8) -> bool {
    (byte >> bit) & 1 == 1
}

pub fn u16_to_vec_u8(v: u16) -> Vec<u8> {
    v.to_le_bytes().to_vec()
}

/// The byte as a printable ASCII character, or a space if it isn't printable
pub fn char_width_one(val: u8) -> char {
    if (0x20..0x7f).contains(&val) {
        val as char
    } else {
        ' '
    }
}

/// Little-endian `u16` from the first two bytes of `v`, missing bytes are treated as zero
pub fn vec_u8_to_u16(v: &[u8]) -> u16 {
    match *v {
        [] => 0,
        [lo] => lo.into(),
        [lo, hi, ..] => u16::from_le_bytes([lo, hi]),
    }
}

#[cfg(test)]
pub mod test {
    use std::path::{Path, PathBuf};

    pub fn rsc<P: AsRef<Path>>(filename: P) -> PathBuf {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("rsc");
        d.push(filename);
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u16_to_vec_u8_tests() {
        let v = u16_to_vec_u8(0xdead);
        assert_eq!(v[0], 0xad);
        assert_eq!(v[1], 0xde);
    }

    #[test]
    fn vec_u8_to_u16_tests() {
        assert_eq!(vec_u8_to_u16(&[0xad, 0xde]), 0xdead);
        assert_eq!(vec_u8_to_u16(&[0xad, 0xde, 0xff]), 0xdead);
        assert_eq!(vec_u8_to_u16(&[0xad]), 0xad);
        assert_eq!(vec_u8_to_u16(&[]), 0);
    }
}
