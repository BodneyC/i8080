use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// use crate::assembler::find_op_code;
use crate::errors::{AssemblerError, GenerationError, OpParseError, ParserError};
use crate::op_meta::I8080_OP_META;

use super::find_op_code;
use super::label::Label;

#[derive(Debug, Clone)]
pub struct LineMeta {
    raw_line: String,
    line_no: usize,
    comment: Option<String>,
    inst: Option<String>,
    args_list: Vec<String>,
    label: Option<String>,
    op_code: Option<u8>,
    label_only: bool,
    address: u16,
    width: usize,
}

impl Default for LineMeta {
    fn default() -> Self {
        Self {
            raw_line: String::new(),
            line_no: 0,
            comment: None,
            inst: None,
            args_list: vec![],
            label: None,
            op_code: None,
            label_only: false,
            address: 0,
            width: 0,
        }
    }
}

impl LineMeta {
    fn erroring(line: &LineMeta) -> Self {
        trace!("@{} | {}", line.line_no, line.raw_line);
        Self {
            line_no: line.line_no,
            raw_line: line.raw_line.to_string(),
            ..Default::default()
        }
    }

    fn label_only(label: Option<String>, raw_line: String) -> Self {
        Self {
            label,
            raw_line,
            label_only: true,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Macro {
    lines: Vec<LineMeta>,
    bytes: Vec<u8>,
    width: usize,
}

impl Macro {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            bytes: Vec::new(),
            width: 0,
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
                    .map_err(|e| AssemblerError::GenerationError(e))
            })
            .and_then(|_| {
                self.generate_bytes()
                    .map_err(|e| AssemblerError::GenerationError(e))
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

    /// Fill the macros with useful bytes...
    ///
    /// Resolve expressions in here
    fn generate_macros(&mut self) -> Result<(), GenerationError> {
        for (_, _macro) in self.macros.borrow_mut().iter_mut() {
            for line in _macro.lines.iter() {
                match self.bytes_for_line(line) {
                    Ok(mut bytes) => {
                        if bytes.len() != line.width {
                            return Err(GenerationError::UnexpectedLength(line.width, bytes.len()));
                        }
                        _macro.bytes.append(&mut bytes);
                    }
                    Err(e) => return Err(GenerationError::ExpressionError(e)),
                }
            }
        }
        Ok(())
    }

    fn bytes_for_line(&self, line: &LineMeta) -> Result<Vec<u8>, ParserError> {
        todo!();
    }

    /// Because of the ORIG instruction, we preallocate the vec so cannot `.append` and must instead
    /// `[line.address]` into it
    fn generate_bytes(&mut self) -> Result<Vec<u8>, GenerationError> {
        let bytes = vec![0; self.highest_address as usize];
        todo!();
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
                        let cond = match self.parse_expression_u16(line.args_list[0].to_string()) {
                            Ok(cond) => cond > 0,
                            Err(e) => return Err(e),
                        };
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
                            let val = match self.parse_expression_u16(arg) {
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
                    continue;
                }
                "ORIG" => {
                    if let Some(name) = current_macro_name {
                        debug!("@{:<03} ORIG used in MACRO '{}'", line.line_no, name);
                        self.erroring_line = Some(LineMeta::erroring(&line));
                        return Err(ParserError::NotInMacro);
                    }
                    let arg = &line.args_list[0];
                    let new_address = match self.parse_expression_u16(arg.to_string()) {
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
                        match self.width_of_vararg(inst_name.to_string(), &line.args_list) {
                            Ok(_width) => {
                                debug!(
                                    "@{:<03} vararg {:?} of length {} (args: {:?})",
                                    line.line_no, line.inst, _width, line.args_list,
                                );
                                width = _width
                            }
                            Err(e) => {
                                debug!(
                                    "@{:<03} vararg {:?} unable to calculate width (args: {:?})",
                                    line.line_no, line.inst, line.args_list,
                                );
                                return Err(e);
                            }
                        }
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
                        let _width = inst_meta.width();
                        debug!(
                            "@{:<03} {:?} with {} (args: {:?})",
                            line.line_no, line.inst, _width, line.args_list,
                        );
                        width = _width;
                    }
                }
            } else {
                debug!("@{:<03} {:?} should be a macro", line.line_no, line.inst,);

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
            new_line.address = address;
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
                debug!(
                    "@{:<03} adding {} to '{}' macro width",
                    line.line_no, width, name
                );
                let m = macros.get_mut(name).unwrap();
                m.width += width;
                m.lines.push(new_line)
            } else {
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
            return Err(ParserError::NoEndIf);
        }

        if let Some(name) = current_macro_name {
            debug!("@EOF MACRO '{}' without ENDM", name);
            return Err(ParserError::NoEndMacro);
        }

        // // Create space for the macros
        // for (_, v) in macros.iter_mut() {
        //     v.bytes = vec![0; v.width];
        // }

        self.macros = RefCell::new(macros);
        self.lines = RefCell::new(resolved_lines);
        self.highest_address = highest_address;

        Ok(())
    }

    pub fn write_file(&self, output: PathBuf, bytes: Vec<u8>) -> Result<(), io::Error> {
        fs::write(output, &bytes)
    }

    fn parse_expression_u16(&self, exp: String) -> Result<u16, ParserError> {
        match self.parse_expression(&exp) {
            Ok(mut bytes) => {
                if bytes.len() > 2 {
                    // This is a bit debatable and would mean that ORIG 'hello'
                    debug!("expression '{}' result longer than u16: {:?}", exp, bytes);
                } else if bytes.len() < 2 {
                    bytes.push(0x00);
                }
                Ok(bytes[0] as u16 | (bytes[1] as u16) << 8)
            }
            Err(e) => Err(e),
        }
    }

    /// Parses any expression to be used as an argument
    fn parse_expression<S: Into<String>>(&self, exp: S) -> Result<Vec<u8>, ParserError> {
        todo!();
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
        match read_lines(input) {
            Ok(lines) => {
                for (line_no, line_res) in lines.enumerate() {
                    if let Ok(line) = line_res {
                        match self.tokenize(&line) {
                            Ok(line_opt) => {
                                if let Some(mut line_meta) = line_opt {
                                    line_meta.line_no = line_no;
                                    line_vec.push(line_meta);
                                }
                            }
                            Err(e) => {
                                return Err(AssemblerError::ParserError(e));
                            }
                        };
                    }
                }
                // self.all_lines = line_vec;
                Ok(line_vec)
            }
            Err(e) => Err(AssemblerError::FileReadError(e)),
        }
    }

    // A more robust system wouldn't be a bad idea, but as the syntax is fairly simple might as
    // well do it fairly simply
    fn tokenize(&mut self, raw_line: &String) -> Result<Option<LineMeta>, ParserError> {
        let mut line = raw_line.trim();
        let mut comment: Option<String> = None;
        // Check for a comment in the line
        if let Some(comment_idx) = line.find(";") {
            if line.len() > comment_idx {
                comment = Some(line[comment_idx + 1..].trim().to_string());
            }
            line = line[..comment_idx].trim();
        }

        // If no line remains, tokenize is still successful, but no LineMeta is returned
        if line.len() == 0 {
            return Ok(None);
        }

        let mut label: Option<String> = None;
        // Check if some label precedes the instruction
        if let Some(label_idx) = line.find(":") {
            label = Some(line[..label_idx].trim().to_string());
            if line.len() > label_idx {
                line = line[label_idx + 1..].trim();
            } else {
                line = "";
            }
        }

        // If no line remains, line is just a marker for a label
        if line.len() == 0 {
            return Ok(Some(LineMeta::label_only(label, raw_line.to_string())));
        }
        let mut raw_args: Option<String> = None;
        // Look for the space between instruction and args
        if let Some(args_idx) = line.find(" ") {
            // No bounds check here as we trim before
            raw_args = Some(line[args_idx + 1..].trim().to_string());
            line = line[..args_idx].trim();
        }

        let mut args_list: Vec<String> = vec![];
        // If any args exists...
        if let Some(args) = raw_args {
            let mut in_quotes: bool = false;
            let mut char_escaped: bool = false;
            let mut this_arg: String = String::new();
            for (idx, c) in args.chars().enumerate() {
                // Start or stop "being" in a string unless escaped
                if c == '\'' && !char_escaped {
                    in_quotes = !in_quotes;
                }
                // If we are escaping this character, unset it
                if char_escaped {
                    char_escaped = false;
                }
                // Only escape inside a string
                if c == '\\' && in_quotes {
                    char_escaped = true;
                }
                // is_escaped can only be set inside a string
                if c == ',' && !in_quotes {
                    args_list.push(this_arg.trim().to_string());
                    this_arg = String::new();
                } else {
                    // Push to this arg before checking end of string
                    this_arg.push(c);
                }
                if idx == args.len() - 1 {
                    if in_quotes {
                        return Err(ParserError::InvalidSyntax("unterminated string"));
                    }
                    args_list.push(this_arg.trim().to_string());
                }
            }
        }

        // Expression resolution is required to know the true op code, however all op codes of the
        // same instruction are the same width which we can use to work out label addressing for
        // expression parsing
        //
        // E.g. MOV M, A is one byte, as is MOV A, A
        //
        // It is this expression parsing that will give us the true arguments and then the true
        // operation code... in short, `op_code` is a temporary value...
        //
        // *Note*: The NoSuchInstruction could be caused by a macro...
        let op_code: Option<u8>;
        match find_op_code::from_args(line, 0, 0) {
            Ok(op) => op_code = Some(op as u8),
            Err(e) => match e {
                OpParseError::NoSuchInstruction(_) => op_code = None,
                _ => {
                    return Err(ParserError::NoInstructionFound(e));
                }
            },
        };

        Ok(Some(LineMeta {
            comment,
            inst: Some(line.to_string()),
            args_list,
            raw_line: raw_line.to_string(),
            label,
            op_code,
            ..Default::default()
        }))
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

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
