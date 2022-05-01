use std::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// use crate::assembler::find_op_code;
use crate::errors::{AssemblerError, OpParseError, ParserError};
use crate::op_meta::I8080_OP_META;

use super::find_op_code;
use super::label::Label;

#[derive(Debug)]
pub struct LineMeta {
    raw_line: String,
    line_no: usize,
    comment: Option<String>,
    inst: Option<String>,
    args_list: Vec<String>,
    label: Option<String>,
    op_code: Option<u8>,
    label_only: bool,
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
        }
    }
}

impl LineMeta {
    fn erroring(line: &LineMeta) -> Self {
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

pub struct Assembler {
    lines: Vec<LineMeta>,
    labels: HashMap<String, Label>,
    macros: HashMap<String, Vec<u8>>,
    instructions: Vec<LineMeta>,

    erroring_line: Option<LineMeta>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            labels: HashMap::new(),
            macros: HashMap::new(),
            instructions: Vec::new(),
            erroring_line: None,
        }
    }

    // Very side-effect-y, I know
    pub fn assemble(&mut self, input: PathBuf) -> Result<Vec<u8>, AssemblerError> {
        self.parse_file(input)
            .and_then(|_| self.validate().map_err(|e| AssemblerError::ParseError(e)))
            .and_then(|_| self.load())
            .map_err(|e| {
                match e.borrow() {
                    AssemblerError::ParseError(e) => {
                        if let Some(line) = &self.erroring_line {
                            self.print_line_err(line, e);
                        } else {
                            self.print_file_err(e);
                        }
                    }
                    _ => println!("error during assembly: {}", e),
                };
                e
            })
    }

    fn load(&mut self) -> Result<Vec<u8>, AssemblerError> {
        todo!()
    }

    fn validate(&mut self) -> Result<(), ParserError> {
        let mut labels: HashMap<String, Label> = HashMap::new();
        let mut macros: HashMap<String, Vec<u8>> = HashMap::new();

        let mut in_macro: bool = false;
        let mut in_if: bool = false;

        for line in self.lines.iter() {
            if let Some(label) = &line.label {
                // Check if we're overwriting a label on any op but SET
                if labels.contains_key(label) {
                    if line.inst.is_none() || line.inst.as_ref().unwrap().as_str() != "SET" {
                        self.erroring_line = Some(LineMeta::erroring(line));
                        return Err(ParserError::LabelAlreadyDefined(
                            label.to_string(),
                            labels.get(label).unwrap().clone(),
                        ));
                    }
                }

                // Load it in without setting a value
                if !line.label_only {
                    match line.inst.as_ref().unwrap().as_str() {
                        "SET" => {
                            labels.insert(label.to_string(), Label::new_set(0));
                        }
                        "EQ" => {
                            labels.insert(label.to_string(), Label::new_eq(0));
                        }
                        "MACRO" => {
                            macros.insert(label.to_string(), vec![]);
                        }
                        _ => {
                            labels.insert(label.to_string(), Label::new(0));
                        }
                    };
                } else {
                    labels.insert(label.to_string(), Label::new(0));
                }
            }

            if line.label_only {
                continue;
            }

            let inst_name = line.inst.as_ref().unwrap();

            // If an op_code is present, this is a predefined instruction
            if let Some(op_code) = &line.op_code {
                let inst_meta = I8080_OP_META[*op_code as usize];

                // Check if instructions which requires a label, has one
                if inst_meta.labelled && line.label.is_none() {
                    self.erroring_line = Some(LineMeta::erroring(line));
                    return Err(ParserError::OperationRequiresLabel(inst_name.clone()));
                }

                // Check we have the correct number of arguments
                if inst_meta.asm_arg_count != line.args_list.len() {
                    self.erroring_line = Some(LineMeta::erroring(line));
                    return Err(ParserError::WrongNumberOfArgs(
                        inst_meta.asm_arg_count,
                        line.args_list.len(),
                    ));
                }
            } else {
                // The only other thing it could be is a macro which should take no arguments
                // Check we have the correct number of arguments
                if line.args_list.len() != 0 {
                    self.erroring_line = Some(LineMeta::erroring(line));
                    return Err(ParserError::WrongNumberOfArgs(0, line.args_list.len()));
                }
            }

            match inst_name.as_str() {
                "MACRO" => {
                    in_macro = true;
                }
                "ENDM" => {
                    if !in_macro {
                        self.erroring_line = Some(LineMeta::erroring(line));
                        return Err(ParserError::NotInMacro);
                    }
                    in_macro = false;
                }
                "IF" => {
                    in_if = true;
                }
                "ENDIF" => {
                    if !in_if {
                        self.erroring_line = Some(LineMeta::erroring(line));
                        return Err(ParserError::NotInIf);
                    }
                    in_if = false;
                }
                _ => {}
            }
        }

        if in_if {
            return Err(ParserError::NoEndIf);
        }

        if in_macro {
            return Err(ParserError::NoEndMacro);
        }

        for line in self.lines.iter() {
            // If the line isn't label-only and no op code was found before, the instruction should
            // refer to a macro
            if !line.label_only && line.op_code.is_none() {
                let inst = line.inst.as_ref().unwrap();
                if !macros.contains_key(inst) {
                    return Err(ParserError::NoMacroFound(inst.to_string()));
                }
            }
        }

        self.labels = labels;
        self.macros = macros;

        Ok(())
    }

    pub fn write_file(&self, output: PathBuf, bytes: Vec<u8>) -> Result<(), io::Error> {
        fs::write(output, &bytes)
    }

    fn print_file_err(&self, e: &ParserError) {
        println!("\nError generated at end of file\n\n{}\n", e);
    }

    fn print_line_err(&self, line: &LineMeta, e: &ParserError) {
        println!(
            "\nError generated here\n\n  {}: {}\n\n{}\n",
            line.line_no, line.raw_line, e
        );
    }

    /// Parses any expression to be used as an argument
    fn parse_expression(&self, exp: String) -> u16 {
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

    // fn width_of_instruction(&self, line: &LineMeta) -> Result<usize, ParserError> {
    //     if let Some(op_code) = &line.op_code {
    //         let op_meta = I8080_OP_META[(*op_code) as usize];
    //         match inst.as_str() {
    //             "DB" => self.db_bytes(line.raw_args),
    //             "DW" => self.dw_bytes(line.raw_args),
    //             "DS" => self.ds_bytes(line.raw_args),
    //             _ => Ok(I8080_OP_META[line.op_code.unwrap() as usize].width()),
    //         }
    //     } else {
    //         Ok(0)
    //     }
    // }

    fn parse_file(&mut self, input: PathBuf) -> Result<(), AssemblerError> {
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
                                return Err(AssemblerError::ParseError(e));
                            }
                        };
                    }
                }
                self.lines = line_vec;
                Ok(())
            }
            Err(e) => Err(AssemblerError::FileReadError(e)),
        }
    }

    // fn load_label(&mut self, line_meta: &LineMeta, addr: u16) -> Result<(), ParserError> {
    //     if line_meta.label.is_some() {
    //         let label = line_meta.label.clone().unwrap();
    //         if let Some(label_address) = self.labels.get(&label) {
    //             Err(ParserError::LabelAlreadyDefined(label, *label_address))
    //         } else {
    //             self.labels.insert(label, addr);
    //             Ok(())
    //         }
    //     } else {
    //         Ok(())
    //     }
    // }

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

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
