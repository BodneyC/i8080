use rand::Rng;

#[derive(Debug)]
pub struct Memory {
    mem: Vec<u8>,
}

const MAX_MEM: usize = 0x10000;

impl Memory {
    pub fn new() -> Self {
        Self {
            mem: vec![0; MAX_MEM],
        }
    }

    /// Copy `prog` into memory at `addr`, truncating anything past the end of memory
    ///
    /// Returns the number of bytes loaded
    pub fn load(&mut self, addr: u16, prog: &[u8]) -> usize {
        let dest = self.mem.get_mut(usize::from(addr)..).unwrap_or_default();
        let len = dest.len().min(prog.len());
        dest[..len].copy_from_slice(&prog[..len]);
        len
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        // Reading unmapped memory is hardware specific, but we'll roll with all ones
        self.mem.get(usize::from(addr)).copied().unwrap_or(0xff)
    }

    pub fn read_word_little_endian(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(addr), self.read_byte(addr.wrapping_add(1))])
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        if let Some(byte) = self.mem.get_mut(usize::from(addr)) {
            *byte = val;
        }
    }

    pub fn write_word_little_endian(&mut self, addr: u16, val: u16) {
        let [lo, hi] = val.to_le_bytes();
        self.write_byte(addr, lo);
        self.write_byte(addr.wrapping_add(1), hi);
    }

    pub(crate) fn randomize(&mut self) {
        rand::thread_rng().fill(&mut self.mem[..]);
    }

    /// Up to `len` bytes from `addr`, stopping at the end of memory
    pub(crate) fn get_slice(&self, addr: u16, len: u16) -> Vec<u8> {
        let start = usize::from(addr).min(self.mem.len());
        let end = (start + usize::from(len)).min(self.mem.len());
        self.mem[start..end].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_full() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(0, &(0..10).collect::<Vec<u8>>());
        for i in 0..10 {
            assert_eq!(mem.read_byte(i), i as u8);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }

    #[test]
    fn load_partial() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(0, &(0..5).collect::<Vec<u8>>());
        for i in 0..5 {
            assert_eq!(mem.read_byte(i), i as u8);
        }
        for i in 5..10 {
            assert_eq!(mem.read_byte(i), 0);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }

    #[test]
    fn load_offset() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(3, &(0..5).collect::<Vec<u8>>());
        for i in 0..3 {
            assert_eq!(mem.read_byte(i), 0);
        }
        for i in 3..8 {
            assert_eq!(mem.read_byte(i), (i - 3) as u8);
        }
        for i in 8..10 {
            assert_eq!(mem.read_byte(i), 0);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }

    #[test]
    fn load_offset_overlapping() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(6, &(0..5).collect::<Vec<u8>>());
        for i in 0..6 {
            assert_eq!(mem.read_byte(i), 0);
        }
        for i in 6..10 {
            assert_eq!(mem.read_byte(i), (i - 6) as u8);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }

    #[test]
    fn past_end_of_memory() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        assert_eq!(mem.load(20, &[1, 2, 3]), 0);
        mem.write_byte(10, 1);
        assert_eq!(mem.get_slice(8, 5), vec![0, 0]);
        assert_eq!(mem.get_slice(20, 5), Vec::<u8>::new());
    }

    #[test]
    fn words_wrap_address_space() {
        let mut mem = Memory::new();
        mem.write_word_little_endian(0xffff, 0xbeef);
        assert_eq!(mem.read_byte(0xffff), 0xef);
        assert_eq!(mem.read_byte(0x0000), 0xbe);
        assert_eq!(mem.read_word_little_endian(0xffff), 0xbeef);
    }
}
