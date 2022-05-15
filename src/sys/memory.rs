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

    pub fn load(&mut self, addr: u16, prog: Vec<u8>) -> usize {
        let offset: usize = addr as usize;
        let mem_len: usize = self.mem.len();
        if offset + prog.len() > self.mem.len() {
            self.mem[offset..mem_len].copy_from_slice(&prog[..mem_len - offset]);
            mem_len - offset
        } else {
            self.mem[offset..offset + prog.len()].copy_from_slice(&prog[..]);
            prog.len()
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match self.mem.get(addr as usize) {
            Some(val) => *val,
            None => 0xff, // This is hardware specific, but we'll roll with it
        }
    }

    pub fn read_word_big_endian(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) << 8 | self.read_byte(addr + 1) as u16
    }

    pub fn read_word_little_endian(&self, addr: u16) -> u16 {
        (self.read_byte(addr + 1) as u16) << 8 | self.read_byte(addr) as u16
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) {
        let idx: usize = addr as usize;
        if idx <= self.mem.len() {
            self.mem[idx] = val;
        }
    }

    pub fn write_word_big_endian(&mut self, addr: u16, val: u16) {
        self.write_byte(addr, (val >> 8) as u8);
        self.write_byte(addr + 1, (val & 0xff) as u8);
    }

    pub fn write_word_little_endian(&mut self, addr: u16, val: u16) {
        self.write_byte(addr + 1, (val >> 8) as u8);
        self.write_byte(addr, val as u8);
    }

    pub(crate) fn randomize(&mut self) {
        for byte in self.mem.iter_mut() {
            *byte = rand::thread_rng().gen();
        }
    }

    pub(crate) fn get_slice(&self, addr: u16, len: u16) -> Vec<u8> {
        let slice = if addr.wrapping_add(len) < addr {
            &self.mem[addr as usize..]
        } else {
            &self.mem[addr as usize..(addr + len) as usize]
        };
        slice.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_full() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(0, (0..10).collect());
        for i in 0..10 {
            assert_eq!(mem.read_byte(i), i as u8);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }

    #[test]
    fn load_partial() {
        let mut mem: Memory = Memory { mem: vec![0; 10] };
        mem.load(0, (0..5).collect());
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
        mem.load(3, (0..5).collect());
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
        mem.load(6, (0..5).collect());
        for i in 0..6 {
            assert_eq!(mem.read_byte(i), 0);
        }
        for i in 6..10 {
            assert_eq!(mem.read_byte(i), (i - 6) as u8);
        }
        assert_eq!(mem.read_byte(10), 0xff);
    }
}
