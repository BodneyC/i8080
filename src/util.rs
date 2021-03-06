use std::{
    fs::{self, File},
    io::{self, BufRead, Read},
    path::Path,
};

pub fn is_bit_set(byte: u8, bit: u8) -> bool {
    (byte >> bit) & 1 == 1
}

pub fn u16_to_vec_u8(v: u16) -> Vec<u8> {
    vec![(v & 0xff) as u8, (v >> 8) as u8]
}

pub fn char_width_one(val: u8) -> char {
    if (0x20..0x7f).contains(&val) {
        val as char
    } else {
        ' '
    }
}

pub fn vec_u8_to_u16(v: &[u8]) -> u16 {
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

pub fn read_file_to_vec_u8<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer)?;
    Ok(buffer)
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
        let vec: Vec<u8> = vec![0xad, 0xde];
        let v = vec_u8_to_u16(&vec);
        assert_eq!(v, 0xdead);
    }
}
