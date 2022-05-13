extern crate rand;

use std::{thread, time};

use rand::Rng;

use crate::op_meta::{OpMeta, I8080_OP_META};

use super::{
    device::{RxDevice, TxDevice},
    flags::Flags,
    memory::Memory,
    registers::Registers,
};

use log::Level;

pub struct I8080 {
    registers: Registers,
    flags: Flags,
    memory: Memory,
    cycles: u64,

    pub halted: bool,
    interrupt_flip_flop: bool,
    interrupt_op_code: Option<u8>,
    rx_devices: Vec<RxDevice>,
    tx_devices: Vec<TxDevice>,

    from_time: time::SystemTime,

    pub interactive: bool,
    pub current_state: String,
}

const FREQUENCY: u64 = 2_000_000;
const STEP_MS: u64 = 10;
const CYCLES_PER_STEP: u64 = (FREQUENCY as f64 / (1000_f64 / STEP_MS as f64)) as u64;

impl I8080 {
    pub fn new(rx_devices: Vec<RxDevice>, tx_devices: Vec<TxDevice>) -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            flags: Flags::new(),
            cycles: 0,
            halted: false,
            interrupt_flip_flop: false,
            interrupt_op_code: None,
            rx_devices,
            tx_devices,
            interactive: false,
            current_state: String::new(),
            from_time: time::SystemTime::now(),
        }
    }

    fn sleep_for_hz(&mut self) {
        if self.cycles > CYCLES_PER_STEP {
            self.cycles -= CYCLES_PER_STEP;
            let d = time::SystemTime::now()
                .duration_since(self.from_time)
                .unwrap();
            let s = u64::from(STEP_MS.saturating_sub(d.as_millis() as u64));
            debug!("CPU: sleep {} millis", s);
            thread::sleep(time::Duration::from_millis(s));
            self.from_time = time::SystemTime::now();
        }
    }

    pub fn run(&mut self, emulate_clock_speed: bool) {
        while !self.halted {
            self.cycle();
            if emulate_clock_speed {
                self.sleep_for_hz();
            }
        }
    }

    pub fn cycle(&mut self) {
        let is_interrupt: bool = self.interrupt_flip_flop && self.interrupt_op_code.is_some();
        let inst: u8 = if is_interrupt {
            self.interrupt_flip_flop = false;
            let inst = self.interrupt_op_code.unwrap();
            self.interrupt_op_code = None;
            inst
        } else {
            self.pc_inst()
        };
        let meta: OpMeta = I8080_OP_META[inst as usize];
        self.execute(inst);
        self.cycles += meta.cycles as u64;
        if self.interactive {
            self.current_state = self.fmt_instruction(inst, meta, is_interrupt);
        }
        self.log_cycle(inst, meta, is_interrupt);
        if !is_interrupt {
            if self.registers.pc as u16 + meta.width() as u16 > u8::MAX.into() {
                warn!("PC larger than address space, halting");
                self.halt()
            } else {
                self.registers.pc += meta.width() as u16;
            }
        }
    }

    pub fn issue_interrupt(&mut self, inst: u8) {
        self.interrupt_flip_flop = true;
        self.interrupt_op_code = Some(inst);
    }

    pub fn load(&mut self, addr: u16, prog: Vec<u8>) -> usize {
        self.memory.load(addr, prog)
    }

    pub fn randomize(&mut self) {
        self.flags.from_byte(rand::thread_rng().gen());
        self.registers.randomize();
        self.memory.randomize();
    }

    pub(crate) fn get_memory_slice(&self, addr: u16, len: u16) -> Vec<u8> {
        self.memory.get_slice(addr, len)
    }

    pub(crate) fn get_pc(&self) -> u16 {
        self.registers.pc
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
            op.push_str(&format!(", {:#04x}", self.pc_argb()));
        } else if meta.argw {
            inst_hex.push_str(&format!(" {:04x}", self.pc_argw()));
            op.push_str(&format!(", {:#06x}", self.pc_argw()));
        }
        let address = if is_interrupt {
            "n/a".to_string()
        } else {
            format!("{}", self.registers.pc - meta.width() as u16)
        };
        format!(
            "Inst {{ addr: {}, dis: \"{}\", hex: [{}], interrupt: {} }}",
            address, op, inst_hex, is_interrupt,
        )
    }

    pub(crate) fn debug_flags(&self) -> String {
        format!("{:?}", self.flags)
    }

    pub(crate) fn debug_registers(&self) -> String {
        format!("{:?}", self.registers)
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
}

mod execute;
