use super::*;

impl I8080 {
    pub(crate) fn execute(&mut self, inst: u8) {
        match inst {
            // ------------------------------------------ SPECIALS
            0x27 => self.daa(),
            0x2f => self.registers.a = !self.registers.a, // CMA
            0x37 => self.flags.carry = true,              // STC
            0x3f => self.flags.carry = !self.flags.carry, // CMC
            0xeb => self.xchg(),

            // ------------------------------------------ UNDOC-NOP
            0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 | 0xcb | 0xd9 | 0xdd | 0xed | 0xfd => {}

            // ------------------------------------------ CONTROL
            0x00 => {} // NOP
            0x76 => self.halt(),
            0xf3 => self.interrupt_flip_flop = true,  // DI
            0xfb => self.interrupt_flip_flop = false, // EI

            // ------------------------------------------ LXI
            0x01 => self.registers.set_bc(self.pc_argw()),
            0x11 => self.registers.set_de(self.pc_argw()),
            0x21 => self.registers.set_hl(self.pc_argw()),
            0x31 => self.registers.sp = self.pc_argw(),

            // ------------------------------------------ LOAD/STORE
            0x0a => self.registers.a = self.memory.read_byte(self.registers.get_bc()), // LDAX B
            0x1a => self.registers.a = self.memory.read_byte(self.registers.get_de()), // LDAX D
            0x2a => self.registers.set_hl(self.memory.read_word(self.pc_argw())),      // LHLD
            0x3a => self.registers.a = self.memory.read_byte(self.pc_argw()),          // LDA

            // STx
            0x02 => self
                .memory
                .write_byte(self.registers.get_bc(), self.registers.a), // STAX B
            0x12 => self
                .memory
                .write_byte(self.registers.get_de(), self.registers.a), // STAX D
            0x22 => self
                .memory
                .write_word(self.pc_argw(), self.registers.get_hl()), // SHLD
            0x32 => self.memory.write_byte(self.pc_argw(), self.registers.a), // STA

            // ------------------------------------------ ROTATE
            0x07 => self.rlc(),
            0x0f => self.rrc(),
            0x17 => self.ral(),
            0x1f => self.rar(),

            // ------------------------------------------ DAD
            0x09 => self.dad(self.registers.get_bc()),
            0x19 => self.dad(self.registers.get_de()),
            0x29 => self.dad(self.registers.get_hl()),
            0x39 => self.dad(self.registers.sp),

            // ------------------------------------------ INC

            // INR
            0x04 => self.registers.b = self.inr(self.registers.b),
            0x0c => self.registers.c = self.inr(self.registers.c),
            0x14 => self.registers.d = self.inr(self.registers.d),
            0x1c => self.registers.e = self.inr(self.registers.e),
            0x24 => self.registers.h = self.inr(self.registers.h),
            0x2c => self.registers.l = self.inr(self.registers.l),
            0x34 => {
                let hl: u16 = self.registers.get_hl();
                let incremented: u8 = self.inr(self.memory.read_byte(hl));
                self.memory.write_byte(hl, incremented);
            }
            0x3c => self.registers.a = self.inr(self.registers.a),

            // INX
            0x03 => self
                .registers
                .set_bc(self.registers.get_bc().wrapping_add(1)),
            0x13 => self
                .registers
                .set_de(self.registers.get_de().wrapping_add(1)),
            0x23 => self
                .registers
                .set_hl(self.registers.get_hl().wrapping_add(1)),
            0x33 => self.registers.pc = self.registers.pc.wrapping_add(1),

            // ------------------------------------------ DEC

            // DCR
            0x05 => self.registers.b = self.dcr(self.registers.b),
            0x0d => self.registers.c = self.dcr(self.registers.c),
            0x15 => self.registers.d = self.dcr(self.registers.d),
            0x1d => self.registers.e = self.dcr(self.registers.e),
            0x25 => self.registers.h = self.dcr(self.registers.h),
            0x2d => self.registers.l = self.dcr(self.registers.l),
            0x35 => {
                let hl: u16 = self.registers.get_hl();
                let decremented: u8 = self.dcr(self.memory.read_byte(hl));
                self.memory.write_byte(hl, decremented);
            }
            0x3d => self.registers.a = self.inr(self.registers.a),

            // DCX
            0x0b => self
                .registers
                .set_bc(self.registers.get_bc().wrapping_sub(1)),
            0x1b => self
                .registers
                .set_de(self.registers.get_de().wrapping_sub(1)),
            0x2b => self
                .registers
                .set_hl(self.registers.get_hl().wrapping_sub(1)),
            0x3b => self.registers.pc -= 1,

            // ------------------------------------------ MOV

            // MVI
            0x06 => self.registers.b = self.pc_argb(),
            0x0e => self.registers.c = self.pc_argb(),
            0x16 => self.registers.d = self.pc_argb(),
            0x1e => self.registers.e = self.pc_argb(),
            0x26 => self.registers.h = self.pc_argb(),
            0x2e => self.registers.l = self.pc_argb(),
            0x36 => self
                .memory
                .write_byte(self.registers.get_hl(), self.pc_argb()),
            0x3e => self.registers.a = self.pc_argb(),

            // MOV B, x
            0x40 => self.registers.b = self.registers.b,
            0x41 => self.registers.b = self.registers.c,
            0x42 => self.registers.b = self.registers.d,
            0x43 => self.registers.b = self.registers.e,
            0x44 => self.registers.b = self.registers.h,
            0x45 => self.registers.b = self.registers.l,
            0x46 => self.registers.b = self.memory.read_byte(self.registers.get_hl()),
            0x47 => self.registers.b = self.registers.a,

            // MOV C, x
            0x48 => self.registers.c = self.registers.b,
            0x49 => self.registers.c = self.registers.c,
            0x4a => self.registers.c = self.registers.d,
            0x4b => self.registers.c = self.registers.e,
            0x4c => self.registers.c = self.registers.h,
            0x4d => self.registers.c = self.registers.l,
            0x4e => self.registers.c = self.memory.read_byte(self.registers.get_hl()),
            0x4f => self.registers.c = self.registers.a,

            // MOV D, x
            0x50 => self.registers.d = self.registers.b,
            0x51 => self.registers.d = self.registers.c,
            0x52 => self.registers.d = self.registers.d,
            0x53 => self.registers.d = self.registers.e,
            0x54 => self.registers.d = self.registers.h,
            0x55 => self.registers.d = self.registers.l,
            0x56 => self.registers.d = self.memory.read_byte(self.registers.get_hl()),
            0x57 => self.registers.d = self.registers.a,

            // MOV E, x
            0x58 => self.registers.e = self.registers.b,
            0x59 => self.registers.e = self.registers.c,
            0x5a => self.registers.e = self.registers.d,
            0x5b => self.registers.e = self.registers.e,
            0x5c => self.registers.e = self.registers.h,
            0x5d => self.registers.e = self.registers.l,
            0x5e => self.registers.e = self.memory.read_byte(self.registers.get_hl()),
            0x5f => self.registers.e = self.registers.a,

            // MOV H, x
            0x60 => self.registers.h = self.registers.b,
            0x61 => self.registers.h = self.registers.c,
            0x62 => self.registers.h = self.registers.d,
            0x63 => self.registers.h = self.registers.e,
            0x64 => self.registers.h = self.registers.h,
            0x65 => self.registers.h = self.registers.l,
            0x66 => self.registers.h = self.memory.read_byte(self.registers.get_hl()),
            0x67 => self.registers.h = self.registers.a,

            // MOV L, x
            0x68 => self.registers.l = self.registers.b,
            0x69 => self.registers.l = self.registers.c,
            0x6a => self.registers.l = self.registers.d,
            0x6b => self.registers.l = self.registers.e,
            0x6c => self.registers.l = self.registers.h,
            0x6d => self.registers.l = self.registers.l,
            0x6e => self.registers.l = self.memory.read_byte(self.registers.get_hl()),
            0x6f => self.registers.l = self.registers.a,

            // MOV M, x
            0x70 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.b),
            0x71 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.c),
            0x72 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.d),
            0x73 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.e),
            0x74 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.h),
            0x75 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.l),
            // 0x76 is HLT below
            0x77 => self
                .memory
                .write_byte(self.registers.get_hl(), self.registers.a),

            // MOV A, x
            0x78 => self.registers.a = self.registers.b,
            0x79 => self.registers.a = self.registers.c,
            0x7a => self.registers.a = self.registers.d,
            0x7b => self.registers.a = self.registers.e,
            0x7c => self.registers.a = self.registers.h,
            0x7d => self.registers.a = self.registers.l,
            0x7e => self.registers.a = self.memory.read_byte(self.registers.get_hl()),
            0x7f => self.registers.a = self.registers.a,

            // Jxx
            0xc3 => self.jmp(self.pc_argw(), true), // JMP
            0xc2 => self.jmp(self.pc_argw(), !self.flags.zero), // JNZ
            0xca => self.jmp(self.pc_argw(), self.flags.zero), // JZ
            0xd2 => self.jmp(self.pc_argw(), !self.flags.carry), // JNC
            0xda => self.jmp(self.pc_argw(), self.flags.carry), // JC
            0xe2 => self.jmp(self.pc_argw(), !self.flags.parity), // JPO
            0xea => self.jmp(self.pc_argw(), self.flags.parity), // JPE
            0xf2 => self.jmp(self.pc_argw(), !self.flags.sign), // JP
            0xfa => self.jmp(self.pc_argw(), self.flags.sign), // JM
            0xe9 => self.registers.pc = self.registers.get_hl(), // PCHL

            // Cxx
            0xcd => self.call(self.pc_argw(), None), // CALL
            0xc4 => self.call(self.pc_argw(), Some(!self.flags.zero)), // CNZ
            0xcc => self.call(self.pc_argw(), Some(self.flags.zero)), // CZ
            0xd4 => self.call(self.pc_argw(), Some(!self.flags.carry)), // CNC
            0xdc => self.call(self.pc_argw(), Some(self.flags.carry)), // CC
            0xe4 => self.call(self.pc_argw(), Some(!self.flags.parity)), // CPO
            0xec => self.call(self.pc_argw(), Some(self.flags.parity)), // CPE
            0xf4 => self.call(self.pc_argw(), Some(!self.flags.sign)), // CP
            0xfc => self.call(self.pc_argw(), Some(self.flags.sign)), // CM

            // Rxx
            0xc9 => self.ret(None),                     // RET
            0xc0 => self.ret(Some(!self.flags.zero)),   // RNZ
            0xc8 => self.ret(Some(self.flags.zero)),    // RZ
            0xd0 => self.ret(Some(!self.flags.carry)),  // RNC
            0xd8 => self.ret(Some(self.flags.carry)),   // RC
            0xe0 => self.ret(Some(!self.flags.parity)), // RPO
            0xe8 => self.ret(Some(self.flags.parity)),  // RPE
            0xf0 => self.ret(Some(!self.flags.sign)),   // RP
            0xf8 => self.ret(Some(self.flags.sign)),    // RM

            // ------------------------------------------ ACCUMULATOR
            // AxI (IMMEDIATES)
            0xc6 => self.add(self.pc_argb(), false),
            0xce => self.add(self.pc_argb(), self.flags.carry),
            0xd6 => self.sub(self.pc_argb(), false),
            0xde => self.sub(self.pc_argb(), self.flags.carry),
            0xe6 => self.ana(self.pc_argb()),
            0xee => self.xra(self.pc_argb()),
            0xf6 => self.ora(self.pc_argb()),
            0xfe => self.cmp(self.pc_argb()),

            // ADD
            0x80 => self.add(self.registers.b, false),
            0x81 => self.add(self.registers.c, false),
            0x82 => self.add(self.registers.d, false),
            0x83 => self.add(self.registers.e, false),
            0x84 => self.add(self.registers.h, false),
            0x85 => self.add(self.registers.l, false),
            0x86 => self.add(self.memory.read_byte(self.registers.get_hl()), false),
            0x87 => self.add(self.registers.a, false),

            // ADC
            0x88 => self.add(self.registers.b, self.flags.carry),
            0x89 => self.add(self.registers.c, self.flags.carry),
            0x8a => self.add(self.registers.d, self.flags.carry),
            0x8b => self.add(self.registers.e, self.flags.carry),
            0x8c => self.add(self.registers.h, self.flags.carry),
            0x8d => self.add(self.registers.l, self.flags.carry),
            0x8e => self.add(
                self.memory.read_byte(self.registers.get_hl()),
                self.flags.carry,
            ),
            0x8f => self.add(self.registers.a, self.flags.carry),

            // SUB
            0x90 => self.sub(self.registers.b, false),
            0x91 => self.sub(self.registers.c, false),
            0x92 => self.sub(self.registers.d, false),
            0x93 => self.sub(self.registers.e, false),
            0x94 => self.sub(self.registers.h, false),
            0x95 => self.sub(self.registers.l, false),
            0x96 => self.sub(self.memory.read_byte(self.registers.get_hl()), false),
            0x97 => self.sub(self.registers.a, false),

            // SBB
            0x98 => self.sub(self.registers.b, self.flags.carry),
            0x99 => self.sub(self.registers.c, self.flags.carry),
            0x9a => self.sub(self.registers.d, self.flags.carry),
            0x9b => self.sub(self.registers.e, self.flags.carry),
            0x9c => self.sub(self.registers.h, self.flags.carry),
            0x9d => self.sub(self.registers.l, self.flags.carry),
            0x9e => self.sub(
                self.memory.read_byte(self.registers.get_hl()),
                self.flags.carry,
            ),
            0x9f => self.sub(self.registers.a, self.flags.carry),

            // ANA
            0xa0 => self.ana(self.registers.b),
            0xa1 => self.ana(self.registers.c),
            0xa2 => self.ana(self.registers.d),
            0xa3 => self.ana(self.registers.e),
            0xa4 => self.ana(self.registers.h),
            0xa5 => self.ana(self.registers.l),
            0xa6 => self.ana(self.memory.read_byte(self.registers.get_hl())),
            0xa7 => self.ana(self.registers.a),

            // XRA
            0xa8 => self.xra(self.registers.b),
            0xa9 => self.xra(self.registers.c),
            0xaa => self.xra(self.registers.d),
            0xab => self.xra(self.registers.e),
            0xac => self.xra(self.registers.h),
            0xad => self.xra(self.registers.l),
            0xae => self.xra(self.memory.read_byte(self.registers.get_hl())),
            0xaf => self.xra(self.registers.a),

            // ORA
            0xb0 => self.ora(self.registers.b),
            0xb1 => self.ora(self.registers.c),
            0xb2 => self.ora(self.registers.d),
            0xb3 => self.ora(self.registers.e),
            0xb4 => self.ora(self.registers.h),
            0xb5 => self.ora(self.registers.l),
            0xb6 => self.ora(self.memory.read_byte(self.registers.get_hl())),
            0xb7 => self.ora(self.registers.a),

            // CMP
            0xb8 => self.cmp(self.registers.b),
            0xb9 => self.cmp(self.registers.c),
            0xba => self.cmp(self.registers.d),
            0xbb => self.cmp(self.registers.e),
            0xbc => self.cmp(self.registers.h),
            0xbd => self.cmp(self.registers.l),
            0xbe => self.cmp(self.memory.read_byte(self.registers.get_hl())),
            0xbf => self.cmp(self.registers.a),

            // ------------------------------------------ STACK

            // PUSH
            0xc5 => self.push(self.registers.get_bc()),
            0xd5 => self.push(self.registers.get_de()),
            0xe5 => self.push(self.registers.get_hl()),
            0xf5 => self.push_psw(),

            // POP
            0xc1 => {
                let val: u16 = self.pop();
                self.registers.set_bc(val);
            }
            0xd1 => {
                let val: u16 = self.pop();
                self.registers.set_de(val);
            }
            0xe1 => {
                let val: u16 = self.pop();
                self.registers.set_hl(val);
            }
            0xf1 => self.pop_psw(),

            0xe3 => self.xthl(),
            0xf9 => self.registers.sp = self.registers.get_hl(), // SPHL

            // ------------------------------------------ IO
            0xd3 => self.dev_out(self.pc_argb() as usize, self.registers.a),
            0xdb => self.registers.a = self.dev_in(self.pc_argb() as usize),

            // ------------------------------------------ RESET
            0xc7 => self.call(0x00, None),
            0xcf => self.call(0x08, None),
            0xd7 => self.call(0x10, None),
            0xdf => self.call(0x18, None),
            0xe7 => self.call(0x20, None),
            0xef => self.call(0x28, None),
            0xf7 => self.call(0x30, None),
            0xff => self.call(0x38, None),
        };
    }

    /// Writes a byte (usually A) to the specified out-device
    ///
    /// If no such in-device exists, nothing is written
    ///
    /// # Arguments
    ///
    /// - val: Value to pass to the out-device
    pub fn dev_out(&self, device: usize, val: u8) {
        if self.tx_devices.len() > device {
            if let Err(err) = self.tx_devices[device].tx.send(val) {
                debug!("error transmitting {} OUT: {:?}", val, err);
            }
        } else {
            debug!("no device OUT");
        }
    }

    /// Read a byte from the specified in-device and writes it to A
    ///
    /// If no such in-device exists, will read all ones, i.e. 0xff
    fn dev_in(&self, device: usize) -> u8 {
        if self.rx_devices.len() > device {
            if let Ok(val) = self.rx_devices[device].rx.try_recv() {
                return val;
            } else {
                debug!("nothing received IN");
            }
        } else {
            debug!("no device IN");
        }
        0xff
    }

    /// Halts the CPU at the end of the current cycle (if using `i8080::run`)
    ///
    /// Though not true to the CPU, sends an end of transmission byte to all out-devices.
    /// This allows the program to end and is akin to a shutdown signal I suppose...
    /// kinda
    pub fn halt(&mut self) {
        self.halted = true;
        for (idx, device) in self.tx_devices.iter().enumerate() {
            self.dev_out(idx, device.eot_byte);
        }
    }

    /// Adds a value to the A register
    ///
    /// This function takes a `carry` argument. If `false` this functions acts
    /// like a [ADD A, x] instruction where as if using the real value of the
    /// `carry` register, this acts like [ADC A, x].
    ///
    /// # Arguments
    ///
    /// - val: Value to add
    /// - carry: Carry flag
    fn add(&mut self, val: u8, carry: bool) {
        let result16: u16 = (self.registers.a as u16)
            .wrapping_add(val as u16)
            .wrapping_add(carry as u16);
        self.flags.carry = result16 > 0xff;
        self.flags.aux_carry = (self.registers.a & 0xf)
            .wrapping_add(val & 0xf)
            .wrapping_add(carry as u8)
            > 0xf;
        self.registers.a = result16 as u8;
        self.flags.zero_sign_parity(self.registers.a);
    }

    /// Subtracts a value from the A register
    ///
    /// This function takes a `carry` argument. If `false` this functions acts
    /// like a [SUB A, x] instruction where as if using the real value of the
    /// `carry` register, this acts like [SBB A, x].
    ///
    /// # Arguments
    ///
    /// - val: Value to subtract
    /// - carry: Carry flag
    fn sub(&mut self, val: u8, carry: bool) {
        self.flags.carry = (self.registers.a as u16) < (val.wrapping_add(carry as u8)) as u16;
        self.flags.aux_carry = (self.registers.a as i8 & 0xf)
            .wrapping_sub(val as i8 & 0xf)
            .wrapping_sub(carry as i8)
            >= 0;
        self.registers.a = ((self.registers.a as u16)
            .wrapping_sub(val as u16)
            .wrapping_sub(carry as u16)) as u8;
        self.flags.zero_sign_parity(self.registers.a);
    }

    /// Performs a logical AND between a given value and A
    ///
    /// # Arguments
    ///
    /// - val: Value to be ANDed with A
    fn ana(&mut self, val: u8) {
        self.flags.carry = false;
        self.flags.aux_carry = ((self.registers.a | val) & 0b1000) != 0;
        self.registers.a &= val;
        self.flags.zero_sign_parity(self.registers.a);
    }

    /// Performs a logical XOR between a given value and A
    ///
    /// # Arguments
    ///
    /// - val: Value to be XORed with A
    fn xra(&mut self, val: u8) {
        self.registers.a ^= val;
        self.flags.carry = false;
        self.flags.aux_carry = false;
        self.flags.zero_sign_parity(self.registers.a);
    }

    /// Performs a logical OR between a given value and A
    ///
    /// # Arguments
    ///
    /// - val: Value to be ORed with A
    fn ora(&mut self, val: u8) {
        self.registers.a |= val;
        self.flags.carry = false;
        self.flags.aux_carry = false;
        self.flags.zero_sign_parity(self.registers.a);
    }

    /// Comapres a value against the A register
    ///
    /// A CMP instruction is essentially a [SUB A, x] but the result is
    /// temporary and discarded. Here, for ease, I am replacing A in
    /// `self.sub(...)` and then restoring it.
    ///
    /// # Arguments
    ///
    /// - val: Value to compare
    fn cmp(&mut self, val: u8) {
        let a: u8 = self.registers.a;
        self.sub(val, false);
        self.registers.a = a;
    }

    /// Increment a value, setting flags
    ///
    /// This instruction does not set the carry flag
    ///
    /// # Arguments
    ///
    /// - val: Value to increment
    fn inr(&mut self, val: u8) -> u8 {
        let result = val.wrapping_add(1);
        self.flags.aux_carry = (result & 0xf).wrapping_add(1) > 0xf;
        self.flags.zero_sign_parity(result);
        result
    }

    /// Decrement a value, setting flags
    ///
    /// This instruction does not set the carry flag
    ///
    /// # Arguments
    ///
    /// - val: Value to decrement
    fn dcr(&mut self, val: u8) -> u8 {
        let result = val.wrapping_sub(1);
        self.flags.aux_carry = (result & 0xf) != 0xf;
        self.flags.zero_sign_parity(result);
        result
    }

    /// Continues execution at a given address based on a condition
    ///
    /// # Arguments
    ///
    /// - addr: Address to which PC should be set
    /// - condition: The condition to test before jumping
    fn jmp(&mut self, addr: u16, condition: bool) {
        if condition {
            self.registers.pc = addr;
        }
    }

    /// Push a value onto the stack
    ///
    /// The order of the bytes *should* be swapped for a true i8080
    /// implementation. However, for ease, I am not doing this.
    ///
    /// # Arguments
    ///
    /// - val: Value to push onto the stack
    fn push(&mut self, val: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(2);
        self.memory.write_word_big_endian(self.registers.sp, val);
    }

    /// Pop a value from the stack
    ///
    /// The order of the bytes *should* be swapped for a true i8080
    /// implementation. However, for ease, I am not doing this.
    ///
    /// # Arguments
    ///
    /// - val: Value to push onto the stack
    fn pop(&mut self) -> u16 {
        let result = self.memory.read_word_big_endian(self.registers.sp);
        self.registers.sp += 2;
        result
    }

    /// Push A and the flags register to the stack
    ///
    /// [A, Flags] -> stack
    fn push_psw(&mut self) {
        let val: u16 = (self.registers.a as u16) << 8 | (self.flags.to_byte() as u16);
        self.push(val);
    }

    /// Push A and the flags register to the stack
    ///
    /// [A, Flags] -> stack
    fn pop_psw(&mut self) {
        let val: u16 = self.pop();
        self.registers.a = (val >> 8) as u8;
        self.flags.from_byte(val as u8);
    }

    /// Exchange the value pointed to by the stack pointer with the value of
    /// the HL register
    fn xthl(&mut self) {
        let indirect: u16 = self.memory.read_word_big_endian(self.registers.sp);
        self.memory
            .write_word_big_endian(self.registers.sp, self.registers.get_hl());
        self.registers.set_hl(indirect);
    }

    /// Exchange the value of HL with DE
    fn xchg(&mut self) {
        let hl: u16 = self.registers.get_hl();
        self.registers.set_hl(self.registers.get_de());
        self.registers.set_de(hl);
    }

    fn call(&mut self, addr: u16, condition: Option<bool>) {
        let mut cond = true;
        if let Some(inner_cond) = condition {
            if inner_cond {
                self.cycles += 6;
            }
            cond = inner_cond;
        }
        if cond {
            self.push(self.registers.pc);
            self.jmp(addr, true);
        }
    }

    fn ret(&mut self, condition: Option<bool>) {
        let mut cond = true;
        if let Some(inner_cond) = condition {
            if inner_cond {
                self.cycles += 6;
            }
            cond = inner_cond;
        }
        if cond {
            self.registers.pc = self.pop();
        }
    }

    /// Decimal Adjust Accumulator, represents the 8-bit value as two, 4-bit,
    /// decimal numbers
    fn daa(&mut self) {
        let mut to_add = 0;
        let mut carry = self.flags.carry;

        let lo = self.registers.a & 0x0f;
        if lo > 9 || self.flags.aux_carry {
            to_add += 0x06;
        }

        let hi = self.registers.a >> 4;
        if hi > 9 || self.flags.carry || (hi >= 9 && lo > 9) {
            to_add += 0x60;
            carry = true;
        }

        self.add(to_add, false);
        self.flags.carry = carry;
    }

    /// Double add, adds a word (a double) to HL
    fn dad(&mut self, val: u16) {
        let hl: u16 = self.registers.get_hl();
        self.flags.carry = hl > 0xffff_u16.wrapping_sub(val);
        self.registers.set_hl(hl.wrapping_add(val));
    }

    /// Rotate A left once, no carry
    fn rlc(&mut self) {
        self.flags.carry = util::is_bit_set(self.registers.a, 7);
        self.registers.a = (self.registers.a << 1) | self.flags.carry as u8;
    }

    /// Rotate A right once, no carry
    fn rrc(&mut self) {
        self.flags.carry = util::is_bit_set(self.registers.a, 0);
        self.registers.a = if self.flags.carry {
            0x80 | (self.registers.a >> 1)
        } else {
            self.registers.a >> 1
        };
    }

    /// Rotate A left once, with carry
    fn ral(&mut self) {
        let carry = util::is_bit_set(self.registers.a, 7);
        self.registers.a = (self.registers.a << 1) | self.flags.carry as u8;
        self.flags.carry = carry;
    }

    /// Rotate A right once, with carry
    fn rar(&mut self) {
        let carry = util::is_bit_set(self.registers.a, 0);
        self.registers.a = if self.flags.carry {
            0x80 | (self.registers.a >> 1)
        } else {
            self.registers.a >> 1
        };
        self.flags.carry = carry;
    }
}

// TODO: Just a shit tonne of tests...
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pop_and_push() {
        let mut i8080 = I8080::new(vec![], vec![]);
        i8080.load(
            0x00,
            vec![
                0x31, 0xff, 0xff, // LXI SP 0x00
                0x01, 0xde, 0xad, // LXI B 0xdead
                0xc5, // PUSH B
                0xd1, // POP D
            ],
        );
        i8080.cycle(); // LXI SP 0x00
        i8080.cycle(); // LXI B 0xdead
        assert_eq!(i8080.registers.get_bc(), 0xdead, "BC is 0xdead");
        i8080.cycle(); // PUSH B
        assert_eq!(i8080.registers.sp, 0xfffd, "SP is initial value - 2");
        assert_eq!(
            i8080.memory.read_word_big_endian(i8080.registers.sp),
            0xdead,
            "[SP] is 0xdead"
        );
        i8080.cycle(); // POP D
        assert_eq!(i8080.registers.sp, 0xffff, "SP is initial value");
        assert_eq!(i8080.registers.get_de(), 0xdead, "DE is 0xdead");
    }
}
