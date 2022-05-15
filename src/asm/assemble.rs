//! 8080 assembler
//!
//! There are some parts of this which are a little off-spec however all seem to fucntion as
//! intended... which is nice.
//!
//! There's no sense of PIC here, nor is there a loader to perform similar tasks. If you wish for
//! your code to be loaded at a particular address... you need to tell the assembler at the start.
//!
//! # Meta-instructions
//!
//! Meta-instructions supported include:
//!
//! - `MACRO` and `ENDM` to define macros
//! - `IF` and `ENDIF` to define pre-compilation conditional blocks of code
//! - `DS`, `DB`, and `DW` to define storage
//! - `ORG` to continue assembling at a specific address (with gaps zero-filled)
//! - `EQU` to immutably, and `SET` to mutably, set values
//!
//! ## Macros
//!
//! Macros are short pieces of code that can be called like an instruction, the compiler generates
//! the code for the macro with each invocation.
//!
//! Macro-endm pairs are supported however they cannot be nested.
//!
//! When macros are called upon, they must have already been defined (that is, sequentially in
//! the source file), so:
//!
//! ```asm
//! _m1: MACRO
//!      LXI B, 0x1234
//!      ENDM
//!      _m1
//! ```
//!
//! will work, however
//!
//! ```asm
//!      _m1
//! _m1: MACRO
//!      LXI B, 0x1234
//!      ENDM
//! ```
//!
//! will not.
//!
//! ## If Blocks
//!
//! Ifs can be used to control if blocks are included in the resulting binary; if the condition is
//! more than zero, the block is included. For example,
//!
//! ```asm
//! IF 12 XOR 12   ; Will not be included as the expression resolves to 0
//! LXI B, 0x1234
//! ENDIF
//! ```
//!
//! ```asm
//! IF 12 XOR 13   ; Will be included as the expression resolves to !0
//! LXI B, 0xdead
//! ENDIF
//! ```
//!
//! If-endif pairs are supported however they cannot be nested.
//!
//! If the condition of the IF uses a label, that label must already be defined sequentially in the
//! source file.
//!
//! ## Defines
//!
//! `DS` takes a single expression resolving to an 8-bit (one-byte) value
//!
//! `DW` is variadic and takes multiple expressions resolving to two-byte (16-bit) values - two
//! character strings may be also used here.
//!
//! The DB instruction can be given in a few forms:
//!
//! ```asm
//! DB 0x45           ; define 1 byte of the value 0x45
//! DB 'some-string$' ; define 12 bytes of the ASCII values of the string
//! DB 0x45, 0x52     ; define 2 bytes of the value 0x4552
//! ```
//!
//! # Examples
//!
//! To assemble a file to a specific output file
//!
//! ```sh
//! i8080 asm \
//!     --output ./hello-world.bin \
//!     --register-definitions \
//!     ./rsc/asm/hello-world.asm
//! ```
//!
//! The flag `--register-definitions` is used to include some `EQU` statements which are fairly
//! standard, these are
//!
//! ```asm
//! B: EQU 0
//! C: EQU 1
//! D: EQU 2
//! E: EQU 3
//! H: EQU 4
//! L: EQU 5
//! M: EQU 6
//! A: EQU 7
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io;

use crate::cli::AssembleArgs;
use crate::meta::I8080_OP_META;
use crate::util;

use super::{
    errors::{AssemblerError, CodeGenError, ParserError},
    expressions::parser::{parse_expression, parse_expression_u16, ExprOutput},
    find_op_code,
    label::Label,
    tokenizer::{self, LineMeta},
};

#[derive(Debug, Clone)]
pub struct Macro {
    lines: Vec<LineMeta>,
    bytes: Vec<u8>,
    width: usize,
    uses_pc: bool,
}

impl Macro {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            bytes: Vec::new(),
            width: 0,
            uses_pc: false,
        }
    }
}

fn get_reg_defs() -> HashMap<String, Label> {
    let mut map = HashMap::new();
    map.insert("B".to_string(), Label::new_addr(Some(0)));
    map.insert("C".to_string(), Label::new_addr(Some(1)));
    map.insert("D".to_string(), Label::new_addr(Some(2)));
    map.insert("E".to_string(), Label::new_addr(Some(3)));
    map.insert("H".to_string(), Label::new_addr(Some(4)));
    map.insert("L".to_string(), Label::new_addr(Some(5)));
    map.insert("M".to_string(), Label::new_addr(Some(6)));
    map.insert("A".to_string(), Label::new_addr(Some(7)));
    map
}

pub struct Assembler {
    args: AssembleArgs,
    lines: RefCell<Vec<LineMeta>>,
    macros: RefCell<HashMap<String, Macro>>,
    labels: HashMap<String, Label>,
    prog_width: u16,
    erroring_line: Option<LineMeta>,
}

impl Assembler {
    pub fn new(args: AssembleArgs) -> Self {
        Self {
            args,
            lines: RefCell::new(Vec::new()),
            macros: RefCell::new(HashMap::new()),
            labels: HashMap::new(),
            prog_width: 0,
            erroring_line: None,
        }
    }

    pub fn assemble(&mut self) -> Result<Vec<u8>, AssemblerError> {
        if self.args.register_definitions {
            self.labels = get_reg_defs();
        }
        self.load_file()
            .and_then(|lines| {
                self.parse_at(lines, self.args.load_at)
                    .map_err(|e| e.into())
            })
            .and_then(|_| self.gen_macros().map_err(|e| e.into()))
            .and_then(|_| self.generate_prog().map_err(|e| e.into()))
            .map_err(|e| {
                self.print_err_msg(&e);
                e
            })
    }

    /// Fill the macros with useful bytes
    ///
    /// The only unknown in epxression resolution is PC ($), if this is found in any expression,
    /// the lines and the macro are marked as `uses_pc`
    ///
    /// When the real code comes to load in the macro, they should check if this flag is set and
    /// regenerate the macro code specific to that call - a better fixup method could exist
    /// but that seems like effort I don't have
    fn gen_macros(&mut self) -> Result<(), CodeGenError> {
        self.erroring_line = None;
        for (_, _macro) in self.macros.borrow_mut().iter_mut() {
            for line in _macro.lines.iter_mut() {
                self.erroring_line = Some(LineMeta::erroring(line));
                let (mut bytes, uses_pc) = self.gen_for_line(line, line.address, true)?;
                if bytes.len() != line.width {
                    return Err(CodeGenError::UnexpectedLength(line.width, bytes.len()));
                }
                if uses_pc {
                    line.uses_pc = true;
                }
                _macro.bytes.append(&mut bytes);
                if line.uses_pc {
                    _macro.uses_pc = true;
                }
            }
        }
        Ok(())
    }

    fn gen_for_line(
        &self,
        line: &LineMeta,
        address: u16,
        inside_macro: bool,
    ) -> Result<(Vec<u8>, bool), CodeGenError> {
        trace!("generating code for line {}", line.line_no);
        if let Some(_) = line.op_code {
            self.gen_for_instruction(line, address)
        } else {
            self.gen_for_macro_call(line, address, inside_macro)
        }
    }

    fn gen_for_instruction(
        &self,
        line: &LineMeta,
        address: u16,
    ) -> Result<(Vec<u8>, bool), CodeGenError> {
        let mut uses_pc = false;
        let mut parsed_exprs: Vec<ExprOutput> = Vec::new();
        for arg in line.args_list.iter() {
            let (v, flags) = parse_expression(arg, address, &self.labels)?;
            parsed_exprs.push((v, flags));
            if flags.pc {
                uses_pc = true;
            }
        }
        debug!(
            "@{:<03} parsed args ({}): {:?}",
            line.line_no,
            parsed_exprs.len(),
            parsed_exprs,
        );
        match line.inst.as_ref().unwrap().as_str() {
            "DB" => {
                let mut bytes: Vec<u8> = vec![];
                for (expr, flags) in parsed_exprs.iter() {
                    if flags.string {
                        for byte in expr {
                            bytes.push(*byte);
                        }
                    } else {
                        bytes.push(expr[0]);
                    }
                }
                debug!("@{:<03} DB generated {} bytes", line.line_no, bytes.len());
                trace!("@{:<03} {:?}", line.line_no, bytes);
                Ok((bytes, uses_pc))
            }
            "DW" => {
                let mut bytes: Vec<u8> = vec![];
                for (expr, _) in parsed_exprs.iter() {
                    let pair = util::vec_u8_to_u16(expr);
                    bytes.push(pair as u8);
                    bytes.push((pair >> 8) as u8);
                }
                debug!("@{:<03} DW generated {} bytes", line.line_no, bytes.len());
                trace!("@{:<03} {:?}", line.line_no, bytes);
                Ok((bytes, uses_pc))
            }
            "DS" => {
                let len = util::vec_u8_to_u16(&parsed_exprs[0].0);
                let bytes: Vec<u8> = vec![0; len as usize];
                debug!("@{:<03} DS {} bytes", line.line_no, bytes.len());
                Ok((bytes, uses_pc))
            }
            name => {
                // We know the number of args is correct so we don't care "which" it is
                let sp = parsed_exprs.iter().any(|e| e.1.sp);
                let psw = parsed_exprs.iter().any(|e| e.1.psw);
                let arg0 = if parsed_exprs.len() > 0 {
                    util::vec_u8_to_u16(&parsed_exprs[0].0)
                } else {
                    0
                };
                let arg1 = if parsed_exprs.len() > 1 {
                    util::vec_u8_to_u16(&parsed_exprs[1].0)
                } else {
                    0
                };
                trace!(
                    "@{:<03} {} {{\n  args: {:?},\n  nargs: {},\n  sp: {}, psw: {}\n}}",
                    line.line_no,
                    name,
                    parsed_exprs,
                    parsed_exprs.len(),
                    sp,
                    psw
                );
                match find_op_code::from_args_and_sp_psw(name, arg0, arg1, sp, psw) {
                    Ok(code) => {
                        debug!("@{:<03} {} op code {}", line.line_no, name, code);

                        let mut inst_bytes: Vec<u8> = vec![code as u8];

                        let meta = I8080_OP_META[code];
                        if meta.argb || meta.argw {
                            trace!("@{:<03} {} has a programmable argument", line.line_no, name);
                            let idx = meta.asm_arg_count - 1;
                            match parsed_exprs.get(idx) {
                                Some((bytes, _)) => {
                                    if meta.argb {
                                        inst_bytes.push(*bytes.get(0).unwrap());
                                    } else if meta.argw {
                                        inst_bytes.push(*bytes.get(0).unwrap());
                                        inst_bytes.push(*bytes.get(1).unwrap());
                                    }
                                }
                                None => {
                                    return Err(CodeGenError::ParserError(
                                        ParserError::WrongNumberOfArgs(idx, parsed_exprs.len()),
                                    ))
                                }
                            }
                        }

                        trace!("@{:<03} {:?}", line.line_no, inst_bytes);
                        return Ok((inst_bytes, uses_pc));
                    }
                    Err(e) => {
                        debug!("@{:<03} {} unable to find op", line.line_no, name);
                        return Err(CodeGenError::ParserError(ParserError::NoInstructionFound(
                            e,
                        )));
                    }
                }
            }
        }
    }

    fn gen_for_macro_call(
        &self,
        line: &LineMeta,
        address: u16,
        inside_macro: bool,
    ) -> Result<(Vec<u8>, bool), CodeGenError> {
        let macros = self.macros.borrow();
        let macro_name = line.inst.as_ref().unwrap();
        let _macro = macros.get(macro_name).unwrap();
        if _macro.uses_pc {
            trace!("@{:<03} {} macro uses PC ($)", line.line_no, macro_name);
            if inside_macro {
                debug!(
                    "@{:<03} macro called inside macro uses PC ($)",
                    line.line_no
                );
                Err(CodeGenError::ParserError(
                    ParserError::MacroCallInMacroUsesSp,
                ))
            } else {
                let bytes = self.gen_for_macro_at(_macro, address)?;
                Ok((bytes, _macro.uses_pc))
            }
        } else {
            trace!(
                "@{:<03} {} macro doesn't use PC ($)",
                line.line_no,
                macro_name
            );
            Ok((_macro.bytes.clone(), _macro.uses_pc))
        }
    }

    fn gen_for_macro_at(&self, _macro: &Macro, addr: u16) -> Result<Vec<u8>, CodeGenError> {
        let mut bytes: Vec<u8> = Vec::new();
        for line in _macro.lines.iter() {
            let (mut _bytes, _) = self.gen_for_line(line, line.address + addr, true)?;
            bytes.append(&mut _bytes);
        }
        Ok(bytes)
    }

    /// Because of the ORG instruction, we preallocate the vec so cannot `.append` and must instead
    /// `[line.address]` into it
    fn generate_prog(&mut self) -> Result<Vec<u8>, CodeGenError> {
        self.erroring_line = None;
        let mut bytes = vec![0; self.prog_width as usize];
        for line in self.lines.borrow().iter() {
            self.erroring_line = Some(LineMeta::erroring(&line));
            let (line_bytes, _) = self.gen_for_line(line, line.address, false)?;
            if line_bytes.len() != line.width {
                return Err(CodeGenError::UnexpectedLength(line.width, bytes.len()));
            }
            for (idx, byte) in line_bytes.iter().enumerate() {
                bytes[line.address as usize + idx] = *byte;
            }
        }
        if self.args.hlt {
            bytes.push(0x76);
        }
        Ok(bytes)
    }

    #[cfg(test)]
    fn parse(&mut self, lines: Vec<LineMeta>) -> Result<(), ParserError> {
        self.parse_at(lines, 0)
    }

    /// Holy shit, this does a lot. And might be the single worst function I've ever written...
    ///
    /// ## Notes
    ///
    /// `self.labels` is populated in place implying that labels cannot be used in expressions
    /// before assignment
    ///
    /// IFs conditions are evaluated implying the same as before, if the condition is `false`, then
    /// the following lines are not loaded until an ENDIF is found
    ///
    /// Macros are tracked, their width and the related lines are recorded, at the end of the
    /// function, width used in vector creation in `self`
    ///
    /// The address is incremented by the width of the instruction or the width of the macro
    /// where appropriate (assumes macro already exists)
    ///
    /// Some expressions are calculated here (earlier than I would like):
    /// - IF, DB, DW, DS, ORG, as they are needed to control the current address
    /// - EQU, SET, as they *may* be needed in the above
    ///
    /// Lines without op codes are macro invocations, these will not be be passed to the
    /// expression parser
    ///
    /// ## Things Done Here
    ///
    /// - Addresses are calculated
    /// - Instruction widths are known
    /// - Macros are sized
    /// - IFs' conditions are known
    /// - The relevant lines are separated from the rest
    fn parse_at(&mut self, lines: Vec<LineMeta>, load_address: u16) -> Result<(), ParserError> {
        let mut resolved_lines: Vec<LineMeta> = Vec::new();

        let mut macros: HashMap<String, Macro> = HashMap::new();
        let mut current_macro_name: Option<&str> = None;

        let mut inside_if: bool = false;
        let mut skip_in_true_if: bool = false;

        let mut address: u16 = load_address;
        let mut macro_address: u16 = load_address;
        let mut highest_address: u16 = load_address;

        for line in lines.iter() {
            self.erroring_line = Some(LineMeta::erroring(line));
            // The IFs control `skip`, if an IF resolves to `false`, we skip until ENDIF
            if let Some(inst) = &line.inst {
                match inst.as_str() {
                    "IF" => {
                        if inside_if {
                            debug!("@{:<03} IF found without previous ENDIF", line.line_no);
                            return Err(ParserError::NestedIf);
                        }
                        let first_arg = line.args_list[0].to_string();
                        let (val, _) = parse_expression_u16(first_arg, address, &self.labels)?;
                        let cond = val > 0;
                        debug!("@{:<03} IF condition is {}", line.line_no, cond);
                        inside_if = true;
                        skip_in_true_if = !cond;
                    }
                    "ENDIF" => {
                        if !inside_if {
                            debug!("@{:<03} ENDIF found without previous IF", line.line_no);
                            return Err(ParserError::NotInIf);
                        }
                        debug!("@{:<03} ENDIF reached", line.line_no);
                        inside_if = false;
                        skip_in_true_if = false;
                    }
                    "END" => {
                        debug!("@{:<03} END found, leaving parser", line.line_no);
                        break;
                    }
                    _ => {}
                }
            }

            if skip_in_true_if {
                continue;
            }

            if let Some(label) = &line.label {
                debug!("@{:<03} contains label ({:?})", line.line_no, line.label);

                // Check if we're overwriting a label on any op but SET
                if self.labels.contains_key(label) {
                    debug!(
                        "@{:<03} label ({:?}) on inst {:?} already loaded",
                        line.line_no, line.inst, line.label
                    );
                    if line.inst.as_ref().unwrap().as_str() != "SET" {
                        return Err(ParserError::LabelAlreadyDefined(
                            label.to_string(),
                            self.labels.get(label).unwrap().clone(),
                        ));
                    }
                }

                // Load it labels without setting a value
                if !line.label_only {
                    let inst = line.inst.as_ref().unwrap().as_str();
                    match inst {
                        // We're checking the use of EQU multiple times above
                        "SET" | "EQU" => {
                            let arg = line.args_list[0].to_string();
                            let (val, _) = parse_expression_u16(arg, address, &self.labels)?;
                            debug!(
                                "@{:<03} label ({:?}) loaded with val ({}) for {:?}",
                                line.line_no, line.label, val, line.inst
                            );
                            if inst == "EQU" {
                                self.labels
                                    .insert(label.to_string(), Label::new_equ(Some(val)));
                            } else {
                                self.labels
                                    .insert(label.to_string(), Label::new_set(Some(val)));
                            }
                        }
                        // This includes MACRO as macro labels can be used in expressions
                        _ => {
                            debug!(
                                "@{:<03} label ({:?}) loaded with addr ({})",
                                line.line_no, line.label, address,
                            );
                            self.labels
                                .insert(label.to_string(), Label::new_addr(Some(address)));
                        }
                    };
                } else {
                    debug!(
                        "@{:<03} label-only ({:?}) loaded with addr ({})",
                        line.line_no, line.label, address,
                    );
                    self.labels
                        .insert(label.to_string(), Label::new_addr(Some(address)));
                }
            }

            if line.label_only {
                let mut new_line = line.clone();
                new_line.address = address;
                new_line.width = 0;
                if let Some(name) = current_macro_name {
                    let m = macros.get_mut(name).unwrap();
                    m.lines.push(new_line)
                } else {
                    resolved_lines.push(new_line);
                }
                continue;
            }

            let width: usize;

            let inst_name = line.inst.as_ref().unwrap();

            // None of these should be in the `new_lines` vec, therefore all `continue`:
            //
            // - Process MACRO start and end
            // - ORG should set the address
            // - IF and ENDIF are processed above
            match inst_name.as_str() {
                "MACRO" => {
                    let label = line.label.as_ref().unwrap();
                    debug!("@{:<03} macro {:?} to be loaded", line.line_no, line.label);
                    if let Some(name) = current_macro_name {
                        debug!("@{:<03} MACRO '{}' found within MACRO", line.line_no, name);
                        return Err(ParserError::NestedMacro);
                    }
                    macros.insert(label.to_string(), Macro::new());
                    current_macro_name = Some(label);
                    continue;
                }
                "ENDM" => {
                    if current_macro_name.is_none() {
                        debug!("@{:<03} ENDM found with no MACRO", line.line_no);
                        return Err(ParserError::NotInMacro);
                    }
                    debug!("@{:<03} ENDM reached", line.line_no);
                    current_macro_name = None;
                    macro_address = load_address;
                    continue;
                }
                "ORG" => {
                    if let Some(name) = current_macro_name {
                        debug!("@{:<03} ORG used in MACRO '{}'", line.line_no, name);
                        return Err(ParserError::OrigInMacro);
                    }
                    let arg = &line.args_list[0];
                    let new_address =
                        match parse_expression_u16(arg.to_string(), address, &self.labels) {
                            Ok((val, _)) => val,
                            Err(e) => {
                                debug!("@{:<03} invalid expr '{}'", line.line_no, arg);
                                return Err(ParserError::ExpressionError(e));
                            }
                        };
                    debug!("@{:<03} ORG with new address {}", line.line_no, new_address);
                    address = new_address;
                    if address > highest_address {
                        highest_address = address;
                    }
                    continue;
                }
                "IF" | "ENDIF" | "SET" | "EQU" => continue,
                _ => {}
            }

            // If an op_code is present, this is a predefined instruction
            if let Some(op_code) = &line.op_code {
                debug!("@{:<03} {:?} is not a macro", line.line_no, line.inst);

                let inst_meta = I8080_OP_META[*op_code as usize];

                // Check if instructions which requires a label, has one
                if inst_meta.labelled && line.label.is_none() {
                    debug!(
                        "@{:<03} {:?} requires a label but none is present",
                        line.line_no, line.inst
                    );
                    return Err(ParserError::OperationRequiresLabel(inst_name.clone()));
                }

                // Check we have the correct number of arguments
                if inst_meta.define {
                    // Check varargs has at least one arg
                    if line.args_list.len() < 1 {
                        debug!(
                            "@{:<03} define {:?} contains no args",
                            line.line_no, line.inst,
                        );
                        return Err(ParserError::NoArgsForVariadic);
                    } else {
                        debug!(
                            "@{:<03} define {:?} with args ({:?})",
                            line.line_no, line.inst, line.args_list,
                        );
                        width =
                            self.width_of_data_storage(inst_name.to_string(), &line.args_list)?;
                    }
                } else {
                    // Check set args has correct #args
                    if inst_meta.asm_arg_count != line.args_list.len() {
                        debug!(
                            "@{:<03} {:?} expected {} args (args: {:?})",
                            line.line_no, line.inst, inst_meta.asm_arg_count, line.args_list,
                        );
                        return Err(ParserError::WrongNumberOfArgs(
                            inst_meta.asm_arg_count,
                            line.args_list.len(),
                        ));
                    } else {
                        debug!(
                            "@{:<03} {:?} with {} (args: {:?})",
                            line.line_no,
                            line.inst,
                            inst_meta.width(),
                            line.args_list,
                        );
                        width = inst_meta.width();
                    }
                }
            } else
            // The only other thing it could be is a macro which should take no arguments
            {
                debug!("@{:<03} {:?} should be a macro", line.line_no, line.inst);

                if line.args_list.len() != 0 {
                    debug!("@{:<03} {:?} macro has arguments", line.line_no, line.inst);
                    return Err(ParserError::WrongNumberOfArgs(0, line.args_list.len()));
                }

                // Check that the macro call is not from within the definition of the macro
                if let Some(name) = current_macro_name {
                    if name == *inst_name {
                        debug!(
                            "@{:<03} {:?} macro is called within its definition",
                            line.line_no, line.inst
                        );
                        return Err(ParserError::RecursiveMacro);
                    }
                }

                // Get the width of the macro as set by this function
                match macros.get(inst_name) {
                    Some(_macro) => {
                        debug!(
                            "@{:<03} macro {:?} is of length {}",
                            line.line_no, line.inst, _macro.width,
                        );
                        width = _macro.width;
                    }
                    None => {
                        debug!("@{:<03} macro {:?} not found", line.line_no, line.inst);
                        return Err(ParserError::MacroUseBeforeCreation);
                    }
                }
            }

            let mut new_line = line.clone();
            new_line.width = width;

            if let Some(name) = current_macro_name {
                // Cannot assign storage in a macro
                match inst_name.as_str() {
                    "DB" | "DW" | "DS" => {
                        debug!("@{:<03} {} found in macro", line.line_no, inst_name);
                        return Err(ParserError::DefineInMacro);
                    }
                    _ => {}
                }
                // Cannot use an IF/ENDIF pair in a macro (implementation, not spec)
                if inside_if {
                    debug!("@{:<03} IF found within macro", line.line_no);
                    return Err(ParserError::IfAndMacroMix);
                }

                new_line.address = macro_address;

                debug!(
                    "@{:<03} adding {} to '{}' macro width",
                    line.line_no, width, name
                );
                let m = macros.get_mut(name).unwrap();
                m.width += width;
                m.lines.push(new_line);

                macro_address += width as u16;
                trace!(
                    "@{:<03}+1 new macro ({}) address: {}",
                    line.line_no,
                    name,
                    address
                );
            } else {
                new_line.address = address;
                debug!("@{:<03} adding to resolved lines", line.line_no);
                resolved_lines.push(new_line);
                // Address doesn't need updating in a macro
                if (address as u32) + (width as u32) > 0xffff {
                    warn!("@{:<03} program wraps the address space", line.line_no);
                }
                address += width as u16;
                trace!("@{:<03}+1 new address: {}", line.line_no, address);
                if address > highest_address {
                    trace!("@{:<03} highest address set", line.line_no);
                    highest_address = address;
                }
            }
        }

        if inside_if {
            debug!("@EOF IFs without ENDIF");
            Err(ParserError::NoEndIf)
        } else if let Some(name) = current_macro_name {
            debug!("@EOF MACRO '{}' without ENDM", name);
            Err(ParserError::NoEndMacro)
        } else {
            self.macros = RefCell::new(macros);
            self.lines = RefCell::new(resolved_lines);
            self.prog_width = highest_address;

            Ok(())
        }
    }

    pub fn write(&self, bytes: Vec<u8>) -> Result<(), io::Error> {
        fs::write(&self.args.output, &bytes)
    }

    fn width_of_data_storage(
        &self,
        inst: String,
        args: &Vec<String>,
    ) -> Result<usize, ParserError> {
        match inst.as_str() {
            "DB" => {
                let mut width = 0;
                for arg in args {
                    let (bytes, flags) = parse_expression(arg, 0, &self.labels)?;
                    if flags.string {
                        width += bytes.len();
                    } else {
                        width += 1;
                    }
                }
                Ok(width)
            }
            "DW" => Ok(args.len() * 2),
            "DS" => {
                let arg0 = args.get(0).unwrap();
                let (bytes, flags) = parse_expression(arg0, 0, &self.labels)?;
                if flags.string {
                    Err(ParserError::InvalidArgument(
                        "DS".to_string(),
                        arg0.to_string(),
                    ))
                } else {
                    let width = bytes.get(0).unwrap();
                    Ok(*width as usize)
                }
            }
            _ => Err(ParserError::UnknownDefine(inst)),
        }
    }

    fn load_file(&mut self) -> Result<Vec<LineMeta>, AssemblerError> {
        self.erroring_line = None;
        let mut line_vec: Vec<LineMeta> = vec![];
        match util::read_lines(&self.args.input) {
            Ok(lines) => {
                for (line_no, line_res) in lines.enumerate() {
                    if let Ok(line) = line_res {
                        self.erroring_line = Some(LineMeta::from_raw(line_no, line.to_string()));
                        let line_opt = tokenizer::tokenize(&line)?;
                        if let Some(mut line_meta) = line_opt {
                            line_meta.line_no = line_no + 1;
                            line_vec.push(line_meta);
                        }
                    }
                }
                Ok(line_vec)
            }
            Err(e) => Err(AssemblerError::FileReadError(e)),
        }
    }

    fn print_err_msg(&mut self, e: &AssemblerError) {
        println!("{}", e);
        if let Some(line) = &self.erroring_line {
            println!("\n  {: <3}| {}", line.line_no, line.raw_line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::path::PathBuf;

    impl AssembleArgs {
        fn new() -> Self {
            Self {
                input: PathBuf::new(),
                output: PathBuf::new(),
                load_at: 0,
                register_definitions: false,
                hlt: false,
            }
        }
    }

    #[test]
    fn width_of_vararg_db() {
        let ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string()];
        let width = ass
            .width_of_data_storage("DB".to_owned(), &args)
            .expect("should parse db arg");
        assert_eq!(width, 1, "a byte of any value should be width 1");

        let args = vec!["13".to_string(), "12".to_string()];
        let width = ass
            .width_of_data_storage("DB".to_owned(), &args)
            .expect("should parse db arg");
        assert_eq!(width, 2, "a list of bytes should return the length");

        let args = vec!["'this-string'".to_string()];
        let width = ass
            .width_of_data_storage("DB".to_owned(), &args)
            .expect("should parse db arg");
        assert_eq!(width, 11, "'this-string' is eleven chars");
    }

    #[test]
    fn width_of_vararg_dw() {
        let ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string(), "fish".to_string()];
        let width = ass
            .width_of_data_storage("DW".to_owned(), &args)
            .expect("should parse db arg");
        assert_eq!(width, 4, "double the numbers of args");
    }

    #[test]
    fn width_of_vararg_ds() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string()];
        let width = ass
            .width_of_data_storage("DS".to_owned(), &args)
            .expect("should parse db arg");
        assert_eq!(width, 13, "the result of the expression");

        let args = vec!["'hello'".to_string()];
        let width = ass.width_of_data_storage("DS".to_owned(), &args);
        assert!(width.is_err(), "should parse db arg");
        let e = width.unwrap_err();
        assert!(matches!(e, ParserError::InvalidArgument(_, _)));
        if let ParserError::InvalidArgument(_, s) = e {
            assert_eq!(s, "'hello'", "erroring arg should be 'hello'");
        }

        ass.labels
            .insert("_ident".to_string(), Label::new_addr(Some(2)));
        let args = vec!["_ident".to_string()];
        let width = ass
            .width_of_data_storage("DS".to_owned(), &args)
            .expect("should parse labels");
        assert_eq!(width, 2, "the result of the expression");
    }

    fn line_meta_for_parse(inst: &str, op: u16, args: Vec<&str>, label: Option<&str>) -> LineMeta {
        LineMeta {
            inst: Some(inst.to_string()),
            op_code: Some(op),
            label: label.map(|l| l.to_string()),
            args_list: args.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn parse_no_meta() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], Some("_l1")),
            line_meta_for_parse("DCR", 0x05, vec!["B"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1234"], None),
            line_meta_for_parse("DS", 0x102, vec!["_l1"], None),
            line_meta_for_parse("ADI", 0xc6, vec!["0x22"], None),
        ];
        let len = raw_lines.len();

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.prog_width, 7);

        let resolved_lines = ass.lines.borrow();
        assert_eq!(resolved_lines.len(), len, "no meta instructions");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.width, 1, "MOV A, B is one byte");
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.width, 1, "DCR B is one byte");
        assert_eq!(l1.address, 1, "DCR B address");

        let l2 = resolved_lines.get(2).unwrap();
        assert_eq!(l2.width, 3, "SHDL takes a u16");
        assert_eq!(l2.address, 2, "SHDL address");

        let l3 = resolved_lines.get(3).unwrap();
        assert_eq!(l3.width, 0, "_l1 refers to address 0");
        assert_eq!(l3.address, 5, "SHDL address");

        let l4 = resolved_lines.get(4).unwrap();
        assert_eq!(l4.width, 2, "ADI takes a u8");
        assert_eq!(l4.address, 5, "DS shouldn't have added to address");
    }

    #[test]
    fn parse_if_endif() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], Some("_l1")),
            line_meta_for_parse("IF", 0x107, vec!["2"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x0000"], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
            line_meta_for_parse("IF", 0x107, vec!["0"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.prog_width, 5);

        let resolved_lines = ass.lines.borrow();

        assert_eq!(resolved_lines.len(), 3, "meta not included");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 1, "IF and ENDIF shouldn't affect address");
        assert_eq!(
            l1.args_list,
            vec!["0x0000".to_string()],
            "SHDL with 0x0000 included"
        );

        let l2 = resolved_lines.get(2).unwrap();
        assert_eq!(l2.address, 4, "continue loading after ENDIF");
    }

    #[test]
    fn parse_nested_if_fails() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("IF", 0x107, vec!["2"], None),
            line_meta_for_parse("IF", 0x107, vec!["0"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
        ];

        let e = ass.parse(raw_lines).expect_err("nested IFs should fail");
        assert!(matches!(e, ParserError::NestedIf));
    }

    #[test]
    fn parse_no_if_for_endif() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![line_meta_for_parse("ENDIF", 0x108, vec![], None)];

        let e = ass
            .parse(raw_lines)
            .expect_err("ENDIF without IF should fail");
        assert!(matches!(e, ParserError::NotInIf));
    }

    #[test]
    fn parse_no_endif_for_if() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![line_meta_for_parse("IF", 0x107, vec!["2"], None)];

        let e = ass
            .parse(raw_lines)
            .expect_err("IF without ENDIF should fail");
        assert!(matches!(e, ParserError::NoEndIf));
    }

    #[test]
    fn parse_macro_endm() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.prog_width, 2);

        let resolved_lines = ass.lines.borrow();

        assert_eq!(resolved_lines.len(), 2, "meta not included");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 1, "continue loading after ENDM");
    }

    #[test]
    fn call_a_macro() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            LineMeta {
                inst: Some("_m1".to_string()),
                ..Default::default()
            },
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.prog_width, 8);

        let resolved_lines = ass.lines.borrow();

        assert_eq!(resolved_lines.len(), 2, "macro not included");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.address, 0, "should be first instruction");
        assert_eq!(l0.width, 5, "should replace with macro width");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 5, "SHDL should be after macro width");
    }

    #[test]
    fn load_address_with_macro() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            LineMeta {
                inst: Some("_m1".to_string()),
                ..Default::default()
            },
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
        ];

        ass.parse_at(raw_lines, 200).expect("should parse");
        assert_eq!(ass.prog_width, 208);

        let resolved_lines = ass.lines.borrow();

        assert_eq!(resolved_lines.len(), 2, "macro not included");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.address, 200, "should be first instruction");
        assert_eq!(l0.width, 5, "should replace with macro width");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 205, "SHDL should be after macro width");
    }

    #[test]
    fn parse_nested_macro_fails() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m2")),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
        ];

        let e = ass.parse(raw_lines).expect_err("nested IFs should fail");
        assert!(matches!(e, ParserError::NestedMacro));
    }

    #[test]
    fn parse_no_macro_for_endm() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![line_meta_for_parse("ENDM", 0x10a, vec![], None)];

        let e = ass
            .parse(raw_lines)
            .expect_err("ENDIF without IF should fail");
        assert!(matches!(e, ParserError::NotInMacro));
    }

    #[test]
    fn parse_no_endm_for_macro() {
        let mut ass = Assembler::new(AssembleArgs::new());
        let raw_lines = vec![line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1"))];
        let e = ass
            .parse(raw_lines)
            .expect_err("IF without ENDIF should fail");
        assert!(matches!(e, ParserError::NoEndMacro));
    }

    #[test]
    fn parse_use_of_org_in_macro() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("ORG", 0x105, vec!["0x0000"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
        ];

        let e = ass
            .parse(raw_lines)
            .expect_err("ORG not permitted in macro");
        assert!(matches!(e, ParserError::OrigInMacro));
    }

    #[test]
    fn parse_org_sets_the_address() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("ORG", 0x105, vec!["0X1000"], None),
            line_meta_for_parse("SHDL", 0x22, vec!["0x1111"], None),
        ];

        ass.parse(raw_lines).expect("should parse");

        let resolved_lines = ass.lines.borrow();
        assert_eq!(resolved_lines.len(), 2, "no meta instructions");

        let l0 = resolved_lines.get(0).unwrap();
        assert_eq!(l0.width, 1, "MOV A, B is one byte");
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.width, 3, "SHDL takes a u16");
        assert_eq!(l1.address, 0x1000, "SHDL address");
    }

    #[test]
    fn parse_label_repeats() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], Some("_l1")),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], Some("_l1")),
        ];

        let err = ass.parse(raw_lines).expect_err("cannot repeat labels");
        assert!(matches!(err, ParserError::LabelAlreadyDefined(_, _)));
        if let ParserError::LabelAlreadyDefined(label, val) = err {
            assert_eq!(label, "_l1", "'_l1' already defined");
            assert_eq!(val.value, Some(0), "previous definition is 0");
        }
    }

    #[test]
    fn parse_equ_no_repeat() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("EQU", 0x103, vec!["200"], Some("label")),
            line_meta_for_parse("EQU", 0x103, vec!["200"], Some("label")),
        ];

        let err = ass.parse(raw_lines).expect_err("cannot repeat EQU");
        assert!(matches!(err, ParserError::LabelAlreadyDefined(_, _)));
        if let ParserError::LabelAlreadyDefined(label, val) = err {
            assert_eq!(label, "label", "'label' already defined");
            assert_eq!(val.value, Some(200), "previous definition is 200");
        }
    }

    #[test]
    fn parse_set_can_be_repeated() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("SET", 0x104, vec!["200"], Some("label")),
            line_meta_for_parse("SET", 0x104, vec!["201"], Some("label")),
        ];

        ass.parse(raw_lines).expect("should parse");

        let labels = ass.labels.borrow();
        assert!(labels.get("label").is_some(), "'label' should exist");

        let label = labels.get("label").unwrap();

        assert_eq!(label.is_set, true, "label is from SET");
        assert_eq!(label.value, Some(201), "label is updated value");
    }

    #[test]
    fn use_of_eq_and_set() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("EQU", 0x103, vec!["200"], Some("EQ200")),
            line_meta_for_parse("SET", 0x104, vec!["201"], Some("SET20X")),
            line_meta_for_parse("SET", 0x104, vec!["EQ200"], Some("SET20X")),
        ];

        ass.parse(raw_lines).expect("should parse");

        let lines = ass.lines.borrow();
        assert_eq!(lines.len(), 0, "no generate-able lines");

        let labels = ass.labels.borrow();

        assert!(labels.get("EQ200").is_some(), "'EQ200' should exist");
        let eq200 = labels.get("EQ200").unwrap();
        assert_eq!(eq200.is_eq, true, "EQ200 is from EQU");
        assert_eq!(eq200.value, Some(200), "EQ200 is updated value");

        assert!(labels.get("SET20X").is_some(), "'SET20X' should exist");
        let set20x = labels.get("SET20X").unwrap();
        assert_eq!(set20x.is_set, true, "SET20X is from SET");
        assert_eq!(set20x.value, Some(200), "SET20X is updated value");
    }

    fn line_meta_for_gen(
        inst: &str,
        op: u16,
        args: Vec<&str>,
        label: Option<&str>,
        address: u16,
        width: usize,
    ) -> LineMeta {
        LineMeta {
            inst: Some(inst.to_string()),
            op_code: Some(op),
            label: label.map(|l| l.to_string()),
            args_list: args.iter().map(|s| s.to_string()).collect(),
            address,
            width,
            ..Default::default()
        }
    }

    #[test]
    fn gen_for_instruction_no_args() {
        let line = line_meta_for_gen("MOV", 0x40, vec!["1", "2"], None, 0, 1);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(!pc, "pc ($) was not used");
        assert_eq!(bytes, vec![0x4a]);
    }

    #[test]
    fn gen_for_instruction_argb() {
        // The line will be loaded with the first MVI (0x06) but it should then resolve to
        // the true OP after expression parsing (0x2e)
        let line = line_meta_for_gen("MVI", 0x06, vec!["5", "0X12"], None, 0, 2);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(!pc, "pc ($) was not used");
        assert_eq!(bytes, vec![0x2e, 0x12]);
    }

    #[test]
    fn gen_for_instruction_argw() {
        let line = line_meta_for_gen("JMP", 0xc3, vec!["0X1234"], None, 0, 3);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(!pc, "pc ($) was not used");
        assert_eq!(bytes, vec![0xc3, 0x34, 0x12]);
    }

    #[test]
    fn gen_for_instruction_argb_with_sp() {
        let line = line_meta_for_gen("IN", 0xdb, vec!["$ + 2"], None, 0x212, 2);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass
            .gen_for_instruction(&line, 0x212)
            .expect("should generate");
        assert!(pc, "pc ($) was used");
        assert_eq!(bytes, vec![0xdb, 0x14]);
    }

    #[test]
    fn gen_for_instruction_db() {
        let line = line_meta_for_gen("DB", 0x100, vec!["0X10", "'ABAB'", "$"], None, 0, 4);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(pc, "pc ($) not used");
        assert_eq!(
            bytes,
            vec![0x10, 'A' as u8, 'B' as u8, 'A' as u8, 'B' as u8, 0x00]
        );
    }

    #[test]
    fn gen_for_instruction_dw() {
        let line = line_meta_for_gen("DW", 0x101, vec!["0X1234", "0X10", "'ABPP'"], None, 0, 6);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(!pc, "pc ($) was not used");
        assert_eq!(bytes, vec![0x34, 0x12, 0x10, 0x00, 'A' as u8, 'B' as u8]);
    }

    #[test]
    fn gen_for_instruction_ds() {
        let line = line_meta_for_gen("DS", 0x102, vec!["10"], None, 0, 10);
        let ass = Assembler::new(AssembleArgs::new());
        let (bytes, pc) = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert!(!pc, "pc ($) was not used");
        assert_eq!(bytes, vec![0x00; 10]);
    }

    fn macro_no_pc() -> Macro {
        Macro {
            lines: vec![
                line_meta_for_gen("MOV", 0x40, vec!["0", "0"], None, 0x00, 1),
                line_meta_for_gen("MOV", 0x40, vec!["0", "1"], None, 0x01, 1),
                line_meta_for_gen("SHDL", 0x22, vec!["0X1111"], None, 0x02, 3),
            ],
            bytes: vec![],
            width: 5,
            uses_pc: false,
        }
    }

    #[test]
    fn gen_macro_no_pc() {
        let mut macros = HashMap::new();
        macros.insert("_m1".to_string(), macro_no_pc());
        let mut ass = Assembler::new(AssembleArgs::new());
        ass.macros = RefCell::new(macros);
        ass.gen_macros().expect("macro should compile");
        let ass_macros = ass.macros.borrow();
        let _m1 = ass_macros
            .get(&"_m1".to_string())
            .expect("macro should exist");
        assert_eq!(_m1.bytes, vec![0x40, 0x41, 0x22, 0x11, 0x11]);
        assert!(!_m1.uses_pc, "shouldn't use PC");
    }

    fn macro_with_pc() -> Macro {
        Macro {
            lines: vec![
                line_meta_for_gen("MOV", 0x40, vec!["0", "0"], None, 0x00, 1),
                line_meta_for_gen("MOV", 0x40, vec!["0", "1"], None, 0x01, 1),
                line_meta_for_gen("SHDL", 0x22, vec!["$"], None, 0x02, 3),
            ],
            bytes: vec![],
            width: 5,
            uses_pc: false,
        }
    }

    #[test]
    fn gen_macro_with_pc() {
        let mut macros = HashMap::new();
        macros.insert("_m1".to_string(), macro_with_pc());
        let mut ass = Assembler::new(AssembleArgs::new());
        ass.macros = RefCell::new(macros);
        ass.gen_macros().expect("macro should compile");
        let ass_macros = ass.macros.borrow();
        let _m1 = ass_macros
            .get(&"_m1".to_string())
            .expect("macro should exist");
        assert_eq!(_m1.bytes, vec![0x40, 0x41, 0x22, 0x02, 0x00]);
        assert!(_m1.uses_pc, "shouldn't use PC");
    }

    #[test]
    fn gen_prog_with_macro_no_pc() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let mut macros = HashMap::new();
        let mut _m1 = macro_no_pc();
        macros.insert("_m1".to_string(), _m1);
        ass.macros = RefCell::new(macros);
        ass.gen_macros().expect("macro should compile");

        let mut labels = HashMap::new();
        labels.insert("B".to_string(), Label::new_equ(Some(0)));
        labels.insert("C".to_string(), Label::new_equ(Some(1)));
        labels.insert("D".to_string(), Label::new_equ(Some(2)));
        ass.labels = labels;

        let lines = vec![
            line_meta_for_gen("MOV", 0x40, vec!["B", "C"], None, 0x00, 1),
            LineMeta {
                inst: Some("_m1".to_string()),
                width: 5,
                address: 0x01,
                ..Default::default()
            },
            line_meta_for_gen("SHDL", 0x22, vec!["$"], None, 0x06, 3),
        ];
        ass.lines = RefCell::new(lines);
        ass.prog_width = 9;

        let bytes = ass.generate_prog().expect("should compile");
        assert_eq!(
            bytes,
            vec![0x41, 0x40, 0x41, 0x22, 0x11, 0x11, 0x22, 0x06, 0x00]
        );
    }

    #[test]
    fn gen_prog_with_macro_with_pc() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let mut macros = HashMap::new();
        let mut _m1 = macro_with_pc();
        macros.insert("_m1".to_string(), _m1);
        ass.macros = RefCell::new(macros);
        ass.gen_macros().expect("macro should compile");

        let mut labels = HashMap::new();
        labels.insert("B".to_string(), Label::new_equ(Some(0)));
        labels.insert("C".to_string(), Label::new_equ(Some(1)));
        labels.insert("D".to_string(), Label::new_equ(Some(2)));
        ass.labels = labels;

        let lines = vec![
            line_meta_for_gen("MOV", 0x40, vec!["B", "C"], None, 0x00, 1),
            LineMeta {
                inst: Some("_m1".to_string()),
                width: 5,
                address: 0x01,
                ..Default::default()
            },
            line_meta_for_gen("SHDL", 0x22, vec!["$"], None, 0x06, 3),
        ];
        ass.lines = RefCell::new(lines);
        ass.prog_width = 9;

        let bytes = ass.generate_prog().expect("should compile");
        assert_eq!(
            bytes,
            vec![
                0x41, 0x40, 0x41, 0x22,
                // The SHDL in the macro is now MOV, MOV, MOV, SHDL, so 0x03
                0x03, 0x00, 0x22, 0x06, 0x00
            ]
        );
    }
}
