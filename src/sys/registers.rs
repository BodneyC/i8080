use rand::Rng;

#[derive(Debug, Default)]
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

/// Register-pair operations are loaded (and thus read) as big endian for some reason
impl Registers {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub(crate) fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub(crate) fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub(crate) fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    pub(crate) fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    pub(crate) fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    pub(crate) fn randomize(&mut self) {
        self.set_bc(rand::thread_rng().gen());
        self.set_de(rand::thread_rng().gen());
        self.set_hl(rand::thread_rng().gen());
        self.a = rand::thread_rng().gen();
        // self.pc = rand::thread_rng().gen();
        self.sp = rand::thread_rng().gen();
    }
}
