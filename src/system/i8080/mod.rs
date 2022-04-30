extern crate rand;

use rand::Rng;

use std::sync::mpsc::{Receiver, Sender};

use crate::op_meta::{OpMeta, I8080_OP_META};
use crate::system::flags::Flags;
use crate::system::memory::Memory;
use crate::system::registers::Registers;
use crate::util;

use log::Level;

pub struct I8080 {
    registers: Registers,
    flags: Flags,
    memory: Memory,
    cycles: u64,

    halted: bool,
    interrupt_flip_flop: bool,
    interrupt_op_code: Option<u8>,
    rx_device: Option<Receiver<u8>>,
    tx_device: Option<Sender<u8>>,
    tx_eot_byte: u8,
}

impl I8080 {
    pub fn new(
        rx_device: Option<Receiver<u8>>,
        tx_device: Option<Sender<u8>>,
        tx_eot_byte: u8,
    ) -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            flags: Flags::new(),
            cycles: 0,
            halted: false,
            interrupt_flip_flop: false,
            interrupt_op_code: None,
            rx_device,
            tx_device,
            tx_eot_byte,
        }
    }

    pub fn run(&mut self) {
        while !self.halted {
            self.cycle();
            // TODO: Timing, Hz, etc.
        }
    }

    pub fn cycle(&mut self) {
        let is_interrupt: bool = self.interrupt_flip_flop && self.interrupt_op_code.is_some();
        let inst: u8 = if is_interrupt {
            self.interrupt_flip_flop = false;
            self.interrupt_op_code = None;
            self.interrupt_op_code.unwrap()
        } else {
            self.pc_inst()
        };
        let meta: OpMeta = I8080_OP_META[inst as usize];
        self.execute(inst);
        self.cycles += meta.cycles as u64;
        self.log_cycle(inst, meta, is_interrupt);
        if !is_interrupt {
            self.registers.pc += meta.width() as u16;
        }
    }

    // NOTE: Not sure how to split ownership between some thread in `run` and
    //   some caller of this
    pub fn issue_interrupt(&mut self, inst: u8) {
        self.interrupt_flip_flop = true;
        self.interrupt_op_code = Some(inst);
    }

    pub fn load(&mut self, addr: u16, prog: Vec<u8>) -> usize {
        self.memory.load(addr, prog)
    }

    /// Get the instruction at PC
    fn pc_inst(&self) -> u8 {
        self.memory.read_byte(self.registers.pc)
    }

    /// Get the byte argument at PC+1
    fn pc_argb(&self) -> u8 {
        self.memory.read_byte(self.registers.pc + 1)
    }

    /// Get the word argument at PC+1
    fn pc_argw(&self) -> u16 {
        self.memory.read_word(self.registers.pc + 1)
    }

    fn fmt_instruction(&self, inst: u8, meta: OpMeta, is_interrupt: bool) -> String {
        let mut inst_hex: String = format!("{:02x}", inst);
        let mut op: String = meta.op.to_owned();
        if meta.argb {
            inst_hex.push_str(&format!(" {:02x}", self.pc_argb()));
            op.push_str(&format!(" {:#04x}", self.pc_argb()));
        } else if meta.argw {
            inst_hex.push_str(&format!(" {:04x}", self.pc_argw()));
            op.push_str(&format!(" {:#06x}", self.pc_argw()));
        }
        format!(
            "Inst {{ cycle: {}, dis: '{}', hex: [{}], interrupt: {} }}",
            self.cycles, op, inst_hex, is_interrupt,
        )
    }

    fn log_components(&self) {
        if log_enabled!(Level::Trace) {
            trace!("{:?}", self.registers);
            trace!("{:?}", self.flags);
        }
    }

    fn log_cycle(&self, inst: u8, meta: OpMeta, is_interrupt: bool) {
        if log_enabled!(Level::Debug) {
            debug!("{}", self.fmt_instruction(inst, meta, is_interrupt));
            self.log_components();
        }
    }

    pub(crate) fn randomize(&mut self) {
        self.flags.from_byte(rand::thread_rng().gen());
        self.registers.randomize();
        self.memory.randomize();
    }
}

mod execute;
