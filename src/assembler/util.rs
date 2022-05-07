use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn u16_to_vec_u8(v: u16) -> Vec<u8> {
    vec![(v & 0xff) as u8, (v >> 8) as u8]
}

pub fn vec_u8_to_u16(v: &Vec<u8>) -> u16 {
    if v.is_empty() {
        0x00
    } else if v.len() == 1 {
        v[0] as u16
    } else {
        v[0] as u16 | (v[1] as u16) << 8
    }
}

pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
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
        let vec: Vec<u8> = vec![0xad, 0xde];
        let v = vec_u8_to_u16(&vec);
        assert_eq!(v, 0xdead);
    }
}
