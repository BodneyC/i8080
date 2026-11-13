use std::{
    thread,
    time::{Duration, Instant},
};

use rand::Rng;

use crate::{
    meta::{OpMeta, I8080_OP_META},
    util,
};

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
    /// Address of the instruction currently executing, PC already points to the next one
    inst_addr: u16,

    pub halted: bool,
    interrupts_enabled: bool,
    interrupt_op_code: Option<u8>,
    rx_devices: Vec<RxDevice>,
    tx_devices: Vec<TxDevice>,

    from_time: Instant,

    pub interactive: bool,
    pub current_state: String,
}

const FREQUENCY: u64 = 2_000_000;
const STEP_MS: u64 = 10;
const CYCLES_PER_STEP: u64 = FREQUENCY * STEP_MS / 1000;

impl I8080 {
    pub fn new(rx_devices: Vec<RxDevice>, tx_devices: Vec<TxDevice>) -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            flags: Flags::new(),
            cycles: 0,
            inst_addr: 0,
            halted: false,
            interrupts_enabled: false,
            interrupt_op_code: None,
            rx_devices,
            tx_devices,
            interactive: false,
            current_state: String::new(),
            from_time: Instant::now(),
        }
    }

    fn sleep_for_hz(&mut self) {
        if self.cycles > CYCLES_PER_STEP {
            self.cycles -= CYCLES_PER_STEP;
            let step = Duration::from_millis(STEP_MS);
            let sleep_for = step.saturating_sub(self.from_time.elapsed());
            debug!("CPU: sleep {:?}", sleep_for);
            thread::sleep(sleep_for);
            self.from_time = Instant::now();
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

    /// Fetch and execute a single instruction, or a pending interrupt if interrupts are enabled
    ///
    /// PC is advanced past the instruction before it executes so that jumps, calls, and returns
    /// can overwrite it, and so `CALL` pushes the address of the following instruction. An
    /// interrupt instruction is not fetched from memory so PC is left as is.
    pub fn cycle(&mut self) {
        self.inst_addr = self.registers.pc;
        let interrupt = if self.interrupts_enabled {
            self.interrupt_op_code.take()
        } else {
            None
        };
        let inst = match interrupt {
            Some(inst) => {
                self.interrupts_enabled = false;
                inst
            }
            None => self.memory.read_byte(self.inst_addr),
        };
        let meta: OpMeta = I8080_OP_META[usize::from(inst)];
        if interrupt.is_none() {
            self.registers.pc = self.inst_addr.wrapping_add(meta.width() as u16);
        }
        self.execute(inst);
        self.cycles += u64::from(meta.cycles);
        if self.interactive || log_enabled!(Level::Debug) {
            self.current_state = self.fmt_instruction(inst, meta, interrupt.is_some());
        }
        self.log_cycle();
    }

    /// Queue an instruction to execute once interrupts are enabled (`EI`)
    pub fn issue_interrupt(&mut self, inst: u8) {
        self.interrupt_op_code = Some(inst);
    }

    /// Load a program into memory, returning the number of bytes loaded
    pub fn load(&mut self, addr: u16, prog: &[u8]) -> usize {
        self.memory.load(addr, prog)
    }

    pub fn randomize(&mut self) {
        self.flags.set_from_byte(rand::thread_rng().gen());
        self.registers.randomize();
        self.memory.randomize();
    }

    pub(crate) fn get_memory_slice(&self, addr: u16, len: u16) -> Vec<u8> {
        self.memory.get_slice(addr, len)
    }

    pub(crate) fn get_pc(&self) -> u16 {
        self.registers.pc
    }

    /// Get the byte argument of the current instruction
    fn pc_argb(&self) -> u8 {
        self.memory.read_byte(self.inst_addr.wrapping_add(1))
    }

    /// Get the word argument of the current instruction
    fn pc_argw(&self) -> u16 {
        self.memory
            .read_word_little_endian(self.inst_addr.wrapping_add(1))
    }

    fn fmt_instruction(&self, inst: u8, meta: OpMeta, is_interrupt: bool) -> String {
        let mut inst_hex: String = format!("{:02x}", inst);
        let mut op: String = meta.op.to_owned();
        if meta.argb {
            let argb = self.pc_argb();
            inst_hex.push_str(&format!(" {:02x}", argb));
            if meta.asm_arg_count == 2 {
                op.push(',');
            }
            op.push_str(&format!(" {:#04x}", argb));
        } else if meta.argw {
            let argw = self.pc_argw();
            inst_hex.push_str(&format!(" {:04x}", argw));
            if meta.asm_arg_count == 2 {
                op.push(',');
            }
            op.push_str(&format!(" {:#06x}", argw));
        }
        let address = if is_interrupt {
            "n/a".to_string()
        } else {
            format!("{:#06x}", self.inst_addr)
        };
        format!(
            "Inst {{ addr: {}, dis: \"{}\", hex: [{}], interrupt: {} }}",
            address, op, inst_hex, is_interrupt,
        )
    }

    fn log_components(&self) {
        if log_enabled!(Level::Trace) {
            trace!("{:?}", self.registers);
            trace!("{:?}", self.flags);
        }
    }

    fn log_cycle(&self) {
        if log_enabled!(Level::Debug) {
            debug!("{}", self.current_state);
            self.log_components();
        }
    }

    /// Example of the output as the format string is not particularly clear...:
    ///
    ///    ┌─────────────────────────┐      ┌────────────┐
    /// PC │ 0x0000                  │      │ Sign:   0  │
    ///    ├─────────────────────────┤      ├────────────┤
    /// SP │ 0xba5f                  │      │ Zero:   0  │
    ///    ├────────────┬────────────┤      ├────────────┤
    /// B  │ 0x63 ('c') │ 0x53 ('S') │ C    │ Aux:    0  │
    ///    ├────────────┼────────────┤      ├────────────┤
    /// D  │ 0xcc (' ') │ 0x48 ('H') │ E    │ Parity: 1  │
    ///    ├────────────┼────────────┤      ├────────────┤
    /// H  │ 0x74 ('t') │ 0x1a (' ') │ L    │ Carry:  0  │
    ///    ├────────────┼────────────┘      └────────────┘
    /// A  │ 0xee (' ') │
    ///    └────────────┘
    pub fn describe_system(&self) -> String {
        format!(
            "   ┌─────────────────────────┐      ┌────────────┐
PC │ {:#06x}                  │      │ Sign:   {}  │
   ├─────────────────────────┤      ├────────────┤
SP │ {:#06x}                  │      │ Zero:   {}  │
   ├────────────┬────────────┤      ├────────────┤
B  │ {:#04x} ('{}') │ {:#04x} ('{}') │ C    │ Aux:    {}  │
   ├────────────┼────────────┤      ├────────────┤
D  │ {:#04x} ('{}') │ {:#04x} ('{}') │ E    │ Parity: {}  │
   ├────────────┼────────────┤      ├────────────┤
H  │ {:#04x} ('{}') │ {:#04x} ('{}') │ L    │ Carry:  {}  │
   ├────────────┼────────────┘      └────────────┘
A  │ {:#04x} ('{}') │
   └────────────┘\
",
            self.registers.pc,
            self.flags.sign as i32,
            self.registers.sp,
            self.flags.zero as i32,
            self.registers.b,
            util::char_width_one(self.registers.b),
            self.registers.c,
            util::char_width_one(self.registers.c),
            self.flags.aux_carry as i32,
            self.registers.d,
            util::char_width_one(self.registers.d),
            self.registers.e,
            util::char_width_one(self.registers.e),
            self.flags.parity as i32,
            self.registers.h,
            util::char_width_one(self.registers.h),
            self.registers.l,
            util::char_width_one(self.registers.l),
            self.flags.carry as i32,
            self.registers.a,
            util::char_width_one(self.registers.a),
        )
    }
}

mod execute;
