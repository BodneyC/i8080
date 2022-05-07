use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io};

use crate::op_meta::I8080_OP_META;

use super::errors::{AssemblerError, CodeGenError, ParserError};
use super::expressions::parser::{parse_expression, ExprOutput};
use super::label::Label;
use super::tokenizer::{self, LineMeta};
use super::{find_op_code, util};

#[derive(Debug)]
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

pub struct Assembler {
    lines: RefCell<Vec<LineMeta>>,
    macros: RefCell<HashMap<String, Macro>>,
    labels: HashMap<String, Label>,
    highest_address: u16,
    erroring_line: Option<LineMeta>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            lines: RefCell::new(Vec::new()),
            macros: RefCell::new(HashMap::new()),
            labels: HashMap::new(),
            highest_address: 0,
            erroring_line: None,
        }
    }

    // Very side-effect-y, I know
    pub fn assemble(&mut self, input: PathBuf) -> Result<Vec<u8>, AssemblerError> {
        self.parse_file(input)
            .and_then(|lines| {
                self.parse(lines)
                    .map_err(|e| AssemblerError::ParserError(e))
            })
            .and_then(|_| {
                self.generate_macros()
                    .map_err(|e| AssemblerError::CodeGenError(e))
            })
            .and_then(|_| {
                self.generate_prog()
                    .map_err(|e| AssemblerError::CodeGenError(e))
            })
            .map_err(|e| {
                match e.borrow() {
                    AssemblerError::ParserError(e) => {
                        if let Some(line) = &self.erroring_line {
                            print_line_err(line, e);
                        } else {
                            print_file_err(e);
                        }
                    }
                    _ => println!("error during assembly: {}", e),
                };
                e
            })
    }

    /// Fill the macros with useful bytes
    ///
    /// The only unknown in epxression resolution is PC ($), if this is found in any expression,
    /// the lines and the macro are marked as `uses_pc`
    ///
    /// When the real code comes to load in the macro, they should check if this flag is set and
    /// regenerate the macro code specific to that call
    fn generate_macros(&mut self) -> Result<(), CodeGenError> {
        for (_, _macro) in self.macros.borrow_mut().iter_mut() {
            for line in _macro.lines.iter_mut() {
                match self.gen_bytes_for_line(line, line.address, true) {
                    Ok((mut bytes, uses_pc)) => {
                        if bytes.len() != line.width {
                            return Err(CodeGenError::UnexpectedLength(line.width, bytes.len()));
                        }
                        if uses_pc {
                            line.uses_pc = true;
                        }
                        _macro.bytes.append(&mut bytes);
                    }
                    Err(e) => {
                        self.erroring_line = Some(LineMeta::erroring(line));
                        return Err(e);
                    }
                }
                if line.uses_pc {
                    _macro.uses_pc = true;
                }
            }
        }
        Ok(())
    }

    fn gen_for_macro_in_place(&self, _macro: &Macro, addr: u16) -> Result<Vec<u8>, CodeGenError> {
        let mut bytes: Vec<u8> = Vec::new();
        for line in _macro.lines.iter() {
            let (mut _bytes, _) = self.gen_bytes_for_line(line, line.address + addr, true)?;
            bytes.append(&mut _bytes);
        }
        Ok(bytes)
    }

    fn gen_bytes_for_instruction(
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
                    "@{:<03} {} {{ args: {:?}, nargs: {}, sp: {}, psw: {} }}",
                    line.line_no,
                    name,
                    line.args_list,
                    parsed_exprs.len(),
                    sp,
                    psw
                );
                match find_op_code::from_args_and_sp_psw(name, arg0, arg1, sp, psw) {
                    Ok(code) => {
                        debug!("@{:<03} {} op code {}", line.line_no, name, code);
                        let mut bytes: Vec<u8> = vec![code as u8; 1];
                        for (expr, _) in parsed_exprs.iter() {
                            let mut arg_bytes = expr.clone();
                            bytes.append(&mut arg_bytes);
                        }
                        trace!("@{:<03} {:?}", line.line_no, bytes);
                        return Ok((bytes, uses_pc));
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

    fn gen_bytes_for_macro_call(
        &self,
        line: &LineMeta,
        address: u16,
        for_macro: bool,
    ) -> Result<(Vec<u8>, bool), CodeGenError> {
        let macros = self.macros.borrow();
        let macro_name = line.inst.as_ref().unwrap();
        let _macro = macros.get(macro_name).unwrap();
        if _macro.uses_pc {
            trace!("@{:<03} {} macro uses PC ($)", line.line_no, macro_name);
            if for_macro {
                debug!(
                    "@{:<03} macro called inside macro uses PC ($)",
                    line.line_no
                );
                Err(CodeGenError::ParserError(
                    ParserError::MacroCallInMacroUsesSp,
                ))
            } else {
                match self.gen_for_macro_in_place(_macro, address) {
                    Ok(bytes) => Ok((bytes, _macro.uses_pc)),
                    Err(e) => Err(e),
                }
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

    fn gen_bytes_for_line(
        &self,
        line: &LineMeta,
        address: u16,
        for_macro: bool,
    ) -> Result<(Vec<u8>, bool), CodeGenError> {
        trace!("generating code for line {}", line.line_no);
        if let Some(_) = line.op_code {
            self.gen_bytes_for_instruction(line, address)
        } else {
            self.gen_bytes_for_macro_call(line, address, for_macro)
        }
    }

    /// Because of the ORIG instruction, we preallocate the vec so cannot `.append` and must instead
    /// `[line.address]` into it
    fn generate_prog(&mut self) -> Result<Vec<u8>, CodeGenError> {
        // self.expression_parser.lexer.labels = self.labels.clone();
        let mut bytes = vec![0; self.highest_address as usize];
        for line in self.lines.borrow().iter() {
            match self.gen_bytes_for_line(line, line.address, false) {
                Ok((line_bytes, _)) => {
                    if line_bytes.len() != line.width {
                        return Err(CodeGenError::UnexpectedLength(line.width, bytes.len()));
                    }
                    for (idx, byte) in line_bytes.iter().enumerate() {
                        bytes[line.address as usize + idx] = *byte;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(bytes)
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
    /// - IF, DB, DW, DS, ORIG, as they are needed to control the current address
    /// - EQ, SET, as they *may* be needed in the above
    ///
    /// END is handled in the next function
    ///
    /// ## Things Done Here
    ///
    /// - Addresses are calculated
    /// - Instruction width are known
    /// - Macros are sized
    /// - IFs' conditions are known
    /// - The relevant lines are separated from the rest
    fn parse(&mut self, lines: Vec<LineMeta>) -> Result<(), ParserError> {
        let mut resolved_lines: Vec<LineMeta> = Vec::new();

        // let mut widths_and_addresses: Vec<(usize, u16)> = Vec::new();

        let mut macros: HashMap<String, Macro> = HashMap::new();
        let mut current_macro_name: Option<&str> = None;

        let mut inside_if: bool = false;
        let mut skip_in_true_if: bool = false;

        let mut address: u16 = 0;
        let mut macro_address: u16 = 0;
        let mut highest_address: u16 = 0;

        for line in lines.iter() {
            // The IFs control `skip`, if an IF resolves to `false`, we skip until ENDIF
            if let Some(inst) = &line.inst {
                match inst.as_str() {
                    "IF" => {
                        if inside_if {
                            debug!("@{:<03} ENDIF found without previous IF", line.line_no);
                            self.erroring_line = Some(LineMeta::erroring(&line));
                            return Err(ParserError::NestedIf);
                        }
                        let first_arg = line.args_list[0].to_string();
                        let cond = self.parse_expr_u16(first_arg, address)? > 0;
                        debug!("@{:<03} IF condition is {}", line.line_no, cond);
                        inside_if = true;
                        skip_in_true_if = !cond;
                    }
                    "ENDIF" => {
                        if !inside_if {
                            debug!("@{:<03} ENDIF found without previous IF", line.line_no);
                            self.erroring_line = Some(LineMeta::erroring(&line));
                            return Err(ParserError::NotInIf);
                        }
                        debug!("@{:<03} ENDIF reached", line.line_no);
                        inside_if = false;
                        skip_in_true_if = false;
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
                    if line.inst.is_none() || line.inst.as_ref().unwrap().as_str() != "SET" {
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::LabelAlreadyDefined(
                            label.to_string(),
                            self.labels.get(label).unwrap().clone(),
                        ));
                    }
                }

                // Load it labels without setting a value
                if !line.label_only {
                    match line.inst.as_ref().unwrap().as_str() {
                        "MACRO" => {
                            debug!("@{:<03} label not loaded for MACRO", line.line_no);
                        }
                        // We're checking the use of EQ multiple times above
                        "SET" | "EQ" => {
                            let arg = line.args_list[0].to_string();
                            let val = match self.parse_expr_u16(arg, address) {
                                Ok(cond) => cond,
                                Err(e) => return Err(e),
                            };
                            debug!(
                                "@{:<03} label ({:?}) loaded with val ({}) for {:?}",
                                line.line_no, line.label, val, line.inst
                            );
                            self.labels
                                .insert(label.to_string(), Label::new_eq(Some(val)));
                        }
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
            // - ORIG should set the address
            // - IF and ENDIF are processed above
            match inst_name.as_str() {
                "MACRO" => {
                    debug!("@{:<03} macro {:?} to be loaded", line.line_no, line.label);
                    if let Some(name) = current_macro_name {
                        debug!("@{:<03} MACRO '{}' found within MACRO", line.line_no, name);
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::NestedMacro);
                    }
                    macros.insert(inst_name.to_string(), Macro::new());
                    current_macro_name = Some(inst_name);
                    continue;
                }
                "ENDM" => {
                    if current_macro_name.is_none() {
                        debug!("@{:<03} ENDM found with no MACRO", line.line_no);
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::NotInMacro);
                    }
                    debug!("@{:<03} ENDM reached", line.line_no);
                    current_macro_name = None;
                    macro_address = 0;
                    continue;
                }
                "ORIG" => {
                    if let Some(name) = current_macro_name {
                        debug!("@{:<03} ORIG used in MACRO '{}'", line.line_no, name);
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::NotInMacro);
                    }
                    let arg = &line.args_list[0];
                    let new_address = match self.parse_expr_u16(arg.to_string(), address) {
                        Ok(cond) => cond,
                        Err(e) => {
                            debug!("@{:<03} invalid expr '{}'", line.line_no, arg);
                            self.erroring_line = Some(LineMeta::erroring(&line));
                            return Err(e);
                        }
                    };
                    debug!(
                        "@{:<03} ORIG with new address {}",
                        line.line_no, new_address
                    );
                    address = new_address;
                    if address > highest_address {
                        highest_address = address;
                    }
                    continue;
                }
                "IF" | "ENDIF" => continue,
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
                    self.erroring_line = Some(LineMeta::erroring(&line));
                    return Err(ParserError::OperationRequiresLabel(inst_name.clone()));
                }

                // Check we have the correct number of arguments
                if inst_meta.varargs {
                    // Check varargs has at least one arg
                    if line.args_list.len() < 1 {
                        debug!(
                            "@{:<03} vararg {:?} contains no args",
                            line.line_no, line.inst,
                        );
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::NoArgsForVariadic);
                    } else {
                        debug!(
                            "@{:<03} vararg {:?} with args ({:?})",
                            line.line_no, line.inst, line.args_list,
                        );
                        width = self.width_of_vararg(inst_name.to_string(), &line.args_list)?;
                    }
                } else {
                    // Check set args has correct #args
                    if inst_meta.asm_arg_count != line.args_list.len() {
                        debug!(
                            "@{:<03} {:?} expected {} args (args: {:?})",
                            line.line_no, line.inst, inst_meta.asm_arg_count, line.args_list,
                        );
                        self.erroring_line = Some(LineMeta::erroring(&line));
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
            } else {
                debug!("@{:<03} {:?} should be a macro", line.line_no, line.inst);

                // The only other thing it could be is a macro which should take no arguments
                if line.args_list.len() != 0 {
                    debug!("@{:<03} {:?} macro has arguments", line.line_no, line.inst);
                    self.erroring_line = Some(LineMeta::erroring(&line));
                    return Err(ParserError::WrongNumberOfArgs(0, line.args_list.len()));
                }

                // Check that the macro call is not from within the definition of the macro
                if let Some(name) = current_macro_name {
                    if name == *inst_name {
                        debug!(
                            "@{:<03} {:?} macro is called within its definition",
                            line.line_no, line.inst
                        );
                        self.erroring_line = Some(LineMeta::erroring(&line));
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
                        self.erroring_line = Some(LineMeta::erroring(&line));
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
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::DefineInMacro);
                    }
                    _ => {}
                }
                // Cannot use an IF/ENDIF pair in a macro (implementation, not spec)
                if inside_if {
                    debug!("@{:<03} IF found within macro", line.line_no);
                    self.erroring_line = Some(LineMeta::erroring(&line));
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
            self.highest_address = highest_address;

            Ok(())
        }
    }

    pub fn write_file(&self, output: PathBuf, bytes: Vec<u8>) -> Result<(), io::Error> {
        fs::write(output, &bytes)
    }

    fn parse_expr_u16(&self, exp: String, addr: u16) -> Result<u16, ParserError> {
        let bytes = self.parse_expr(&exp, addr)?;
        if bytes.len() > 2 {
            // This is a bit debatable and would mean that ORIG 'hello'
            debug!("expression '{}' result longer than u16: {:?}", exp, bytes);
        }
        Ok(util::vec_u8_to_u16(&bytes))
    }

    /// Parses any expression to be used as an argument
    fn parse_expr<S: Into<String>>(&self, exp: S, addr: u16) -> Result<Vec<u8>, ParserError> {
        let (bytes, _) = parse_expression(exp, addr, &self.labels)?;
        Ok(bytes)
    }

    ///// The DB instruction can be given in a few forms:
    /////
    ///// DB 45H            ; define 1 byte of the value 0x45
    ///// DB 'some-string$' ; define 12 bytes of the ASCII values of the string
    ///// DB 45H, 52H       ; define 2 bytes of the value 0x4552
    //fn db_bytes(&self, arg_opt: Option<String>) -> Result<usize, ParserError> {
    //    // Check that arg exist
    //    if let Some(arg) = arg_opt {
    //        // Check that arg is not empty
    //        if arg.len() == 0 {
    //            Err(ParserError::TooFewArguments)
    //        } else if arg.chars().nth(0) == Some('\'') {
    //            // Check that first and last char is '\''
    //            if arg.chars().last() != Some('\'') {
    //                Err(ParserError::InvalidSyntax("unterminated string"))
    //            } else {
    //                Ok(arg.chars().count() - 2 - arg.matches('\\').count())
    //            }
    //        } else {
    //            // If not a string literal, then a comma separated list of bytes
    //            Ok(arg.matches(",").count() + 1)
    //        }
    //    } else {
    //        Err(ParserError::TooFewArguments)
    //    }
    //}

    ///// The DW instruction can be given in a couple of forms:
    /////
    ///// DW 4552H        ; define 2 bytes of the value 0x5245
    ///// DB 4552H, 52adH ; define 4 bytes of the value 0x5245ad52
    //fn dw_bytes(&self, arg_opt: Option<String>) -> Result<usize, ParserError> {
    //    // Check that arg exist
    //    if let Some(arg) = arg_opt {
    //        if arg.len() == 0 {
    //            // Check that arg is not empty
    //            Err(ParserError::TooFewArguments)
    //        } else if arg.chars().nth(0) == Some('\'') {
    //            // Check that arg is a string literal
    //            Err(ParserError::InvalidSyntax("string invalid for DW"))
    //        } else {
    //            // Arg should be a comma separated list of byte pairs
    //            Ok((arg.matches(",").count() + 1) * 2)
    //        }
    //    } else {
    //        Err(ParserError::TooFewArguments)
    //    }
    //}

    // fn ds_bytes(&self, arg_opt: Option<String>) -> Result<usize, ParserError> {}

    fn width_of_vararg(&self, inst: String, args: &Vec<String>) -> Result<usize, ParserError> {
        todo!();
        // match inst.as_str() {
        //     "DB" => self.db_bytes(line.raw_args),
        //     "DW" => self.dw_bytes(line.raw_args),
        //     "DS" => self.ds_bytes(line.raw_args),
        //     _ => Ok(I8080_OP_META[line.op_code.unwrap() as usize].width()),
        // }
    }

    fn parse_file(&mut self, input: PathBuf) -> Result<Vec<LineMeta>, AssemblerError> {
        let mut line_vec: Vec<LineMeta> = vec![];
        match util::read_lines(input) {
            Ok(lines) => {
                for (line_no, line_res) in lines.enumerate() {
                    if let Ok(line) = line_res {
                        let line_opt = tokenizer::tokenize(&line)?;
                        if let Some(mut line_meta) = line_opt {
                            line_meta.line_no = line_no;
                            line_vec.push(line_meta);
                        }
                    }
                }
                // self.all_lines = line_vec;
                Ok(line_vec)
            }
            Err(e) => Err(AssemblerError::FileReadError(e)),
        }
    }
}

fn print_file_err(e: &ParserError) {
    println!("\nError generated at end of file\n\n{}\n", e);
}

fn print_line_err(line: &LineMeta, e: &ParserError) {
    println!(
        "\nError generated here\n\n  {}: {}\n\n{}\n",
        line.line_no, line.raw_line, e
    );
}
