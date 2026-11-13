//! 8080 assembler
//!
//! There are some parts of this which are a little off-spec however all seem to function as
//! intended... which is nice.
//!
//! There's no sense of PIC here, nor is there a loader to perform similar tasks. If you wish for
//! your code to be loaded at a particular address, you need to tell the assembler with
//! `--load-at`; the resulting binary begins at that address.
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
//! - `END` to stop assembling
//!
//! ## Macros
//!
//! Macros are short pieces of code that can be called like an instruction, the assembler
//! generates the code for the macro with each invocation.
//!
//! Macro definitions cannot be nested, though a macro may call other, previously defined, macros.
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
//! `DS` takes a single expression and reserves that many zeroed bytes
//!
//! `DW` is variadic and takes multiple expressions resolving to two-byte (16-bit) values - two
//! character strings may be also used here.
//!
//! The DB instruction can be given in a few forms:
//!
//! ```asm
//! DB 0x45           ; define 1 byte of the value 0x45
//! DB 'some-string$' ; define 12 bytes of the ASCII values of the string
//! DB 0x45, 0x52     ; define 2 bytes, 0x45 followed by 0x52
//! ```
//!
//! # Examples
//!
//! To assemble a file to a specific output file
//!
//! ```sh
//! i8080 asm --output ./hello-world.bin ./rsc/asm/hello-world.asm
//! ```
//!
//! # Register Names
//!
//! By default the register names are defined as if by the following `EQU` statements, the flag
//! `--no-register-definitions` (`--no-reg-defs`) omits them, freeing the names for use as labels
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

use std::collections::HashMap;
use std::fs;
use std::io;

use crate::cli::AssembleArgs;
use crate::meta::I8080_OP_META;
use crate::util;

use super::{
    errors::{AssemblerError, CodeGenError, OpParseError, ParserError},
    expressions::parser::{parse_expression, parse_expression_u16, ExprOutput},
    find_op_code,
    label::{Label, LabelKind},
    tokenizer::{self, LineMeta},
};

/// One past the highest addressable byte
const ADDRESS_SPACE: u32 = 0x10000;

const REGISTERS: [(&str, u16); 8] = [
    ("B", 0),
    ("C", 1),
    ("D", 2),
    ("E", 3),
    ("H", 4),
    ("L", 5),
    ("M", 6),
    ("A", 7),
];

fn register_definitions() -> HashMap<String, Label> {
    REGISTERS
        .iter()
        .map(|&(name, value)| (name.to_string(), Label::new_equ(value)))
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct Macro {
    /// Lines of the macro body, addressed relative to the start of the macro
    lines: Vec<LineMeta>,
    width: usize,
}

/// Mutable state used while walking the source in [`Assembler::parse_at`]
#[derive(Debug)]
struct ParseState {
    load_address: u16,
    /// `u32` so that a program ending exactly at the top of memory is representable
    address: u32,
    highest_address: u32,
    macro_name: Option<String>,
    macro_offset: u16,
    in_if: bool,
    skipping: bool,
}

impl ParseState {
    fn new(load_address: u16) -> Self {
        Self {
            load_address,
            address: load_address.into(),
            highest_address: load_address.into(),
            macro_name: None,
            macro_offset: 0,
            in_if: false,
            skipping: false,
        }
    }

    /// The current address as a valid 8080 address
    fn pc(&self) -> Result<u16, ParserError> {
        u16::try_from(self.address).map_err(|_| ParserError::AddressOverflow)
    }

    fn advance(&mut self, width: usize) -> Result<(), ParserError> {
        self.address = u32::try_from(width)
            .ok()
            .and_then(|width| self.address.checked_add(width))
            .filter(|&address| address <= ADDRESS_SPACE)
            .ok_or(ParserError::AddressOverflow)?;
        self.highest_address = self.highest_address.max(self.address);
        Ok(())
    }
}

fn first_arg(line: &LineMeta) -> Result<&str, ParserError> {
    line.args_list
        .first()
        .map(String::as_str)
        .ok_or(ParserError::WrongNumberOfArgs(1, 0))
}

/// The value of an evaluated expression, which must not be empty (e.g. `''`, `SP`, or `PSW`)
fn expr_value(inst: &str, arg: &str, expr: &[u8]) -> Result<u16, ParserError> {
    if expr.is_empty() {
        Err(ParserError::InvalidArgument(
            inst.to_string(),
            arg.to_string(),
        ))
    } else {
        Ok(util::vec_u8_to_u16(expr))
    }
}

pub struct Assembler {
    args: AssembleArgs,
    /// Lines to generate code for, i.e. instructions, defines, and macro calls
    lines: Vec<LineMeta>,
    macros: HashMap<String, Macro>,
    labels: HashMap<String, Label>,
    /// Address one past the last byte of the program
    end_address: u32,
    /// Line number and source of the line being processed, reported alongside errors
    current_line: Option<(usize, String)>,
}

impl Assembler {
    pub fn new(args: AssembleArgs) -> Self {
        Self {
            args,
            lines: Vec::new(),
            macros: HashMap::new(),
            labels: HashMap::new(),
            end_address: 0,
            current_line: None,
        }
    }

    /// Assemble the input file, returning the program as it should be loaded at `load_at`
    pub fn assemble(&mut self) -> Result<Vec<u8>, AssemblerError> {
        if !self.args.no_register_definitions {
            self.labels = register_definitions();
        }
        self.assemble_inner()
            .map_err(|e| match self.current_line.take() {
                Some((line_no, raw_line)) => AssemblerError::AtLine {
                    line_no,
                    raw_line,
                    source: Box::new(e),
                },
                None => e,
            })
    }

    fn assemble_inner(&mut self) -> Result<Vec<u8>, AssemblerError> {
        let lines = self.load_file()?;
        self.parse_at(lines, self.args.load_at)?;
        Ok(self.generate_prog()?)
    }

    pub fn write(&self, bytes: &[u8]) -> Result<(), io::Error> {
        fs::write(&self.args.output, bytes)
    }

    fn load_file(&mut self) -> Result<Vec<LineMeta>, AssemblerError> {
        let source = fs::read_to_string(&self.args.input)?;
        let mut lines = vec![];
        for (idx, raw_line) in source.lines().enumerate() {
            let line_no = idx + 1;
            self.current_line = Some((line_no, raw_line.to_string()));
            if let Some(mut line) = tokenizer::tokenize(raw_line)? {
                line.line_no = line_no;
                lines.push(line);
            }
        }
        self.current_line = None;
        Ok(lines)
    }

    #[cfg(test)]
    fn parse(&mut self, lines: Vec<LineMeta>) -> Result<(), ParserError> {
        self.parse_at(lines, 0)
    }

    /// First pass over the source
    ///
    /// Labels are defined, `IF`s are evaluated, macros are recorded, and every remaining line is
    /// given its address and width ready for code generation.
    ///
    /// As labels are defined in a single pass, any expression evaluated here (`IF`, `ORG`, `EQU`,
    /// `SET`, `DB`, `DS`) may only use labels defined earlier in the source.
    fn parse_at(&mut self, lines: Vec<LineMeta>, load_address: u16) -> Result<(), ParserError> {
        let mut state = ParseState::new(load_address);
        self.lines.clear();
        self.macros.clear();

        for line in lines {
            self.current_line = Some((line.line_no, line.raw_line.clone()));

            match line.inst.as_deref() {
                Some("IF") => self.parse_if(&line, &mut state)?,
                Some("ENDIF") => {
                    if !state.in_if {
                        return Err(ParserError::NotInIf);
                    }
                    state.in_if = false;
                    state.skipping = false;
                }
                Some("END") if !state.skipping => {
                    debug!("@{:<03} END found, leaving parser", line.line_no);
                    break;
                }
                _ => {}
            }

            if state.skipping {
                continue;
            }

            if let Some(op_code) = line.op_code {
                validate_op(&line, op_code)?;
            }

            if let Some(label) = &line.label {
                self.define_label(label, &line, state.pc()?)?;
            }

            match line.inst.as_deref() {
                None | Some("IF" | "ENDIF" | "SET" | "EQU") => {}
                Some("MACRO") => self.begin_macro(&line, &mut state)?,
                Some("ENDM") => {
                    if state.macro_name.take().is_none() {
                        return Err(ParserError::NotInMacro);
                    }
                    state.macro_offset = 0;
                }
                Some("ORG") => self.parse_org(&line, &mut state)?,
                Some(_) => self.place_line(line, &mut state)?,
            }
        }

        self.current_line = None;

        if state.in_if {
            Err(ParserError::NoEndIf)
        } else if state.macro_name.is_some() {
            Err(ParserError::NoEndMacro)
        } else {
            self.end_address = state.highest_address;
            Ok(())
        }
    }

    fn parse_if(&self, line: &LineMeta, state: &mut ParseState) -> Result<(), ParserError> {
        if state.in_if {
            return Err(ParserError::NestedIf);
        }
        let (cond, _) = parse_expression_u16(first_arg(line)?, state.pc()?, &self.labels)?;
        debug!("@{:<03} IF condition is {}", line.line_no, cond > 0);
        state.in_if = true;
        state.skipping = cond == 0;
        Ok(())
    }

    fn parse_org(&self, line: &LineMeta, state: &mut ParseState) -> Result<(), ParserError> {
        if state.macro_name.is_some() {
            return Err(ParserError::OrgInMacro);
        }
        let (address, _) = parse_expression_u16(first_arg(line)?, state.pc()?, &self.labels)?;
        if address < state.load_address {
            return Err(ParserError::OrgBeforeLoadAddress(
                address,
                state.load_address,
            ));
        }
        debug!(
            "@{:<03} ORG with new address {:#06x}",
            line.line_no, address
        );
        state.address = address.into();
        state.highest_address = state.highest_address.max(state.address);
        Ok(())
    }

    fn begin_macro(&mut self, line: &LineMeta, state: &mut ParseState) -> Result<(), ParserError> {
        if state.macro_name.is_some() {
            return Err(ParserError::NestedMacro);
        }
        let name = line
            .label
            .clone()
            .ok_or_else(|| ParserError::OperationRequiresLabel("MACRO".to_string()))?;
        debug!("@{:<03} defining macro {}", line.line_no, name);
        self.macros.insert(name.clone(), Macro::default());
        state.macro_name = Some(name);
        Ok(())
    }

    /// `SET` labels may be redefined by another `SET`, all other labels are immutable
    fn define_label(&mut self, label: &str, line: &LineMeta, pc: u16) -> Result<(), ParserError> {
        let inst = line.inst.as_deref();
        if let Some(existing) = self.labels.get(label) {
            if inst != Some("SET") || existing.kind != LabelKind::Set {
                return Err(ParserError::LabelAlreadyDefined(
                    label.to_string(),
                    *existing,
                ));
            }
        }
        let value = match inst {
            Some("SET") => {
                Label::new_set(parse_expression_u16(first_arg(line)?, pc, &self.labels)?.0)
            }
            Some("EQU") => {
                Label::new_equ(parse_expression_u16(first_arg(line)?, pc, &self.labels)?.0)
            }
            _ => Label::new_addr(pc),
        };
        debug!("@{:<03} label {} = {:?}", line.line_no, label, value);
        self.labels.insert(label.to_string(), value);
        Ok(())
    }

    /// Size a line which generates code and add it to either the program or the current macro
    fn place_line(
        &mut self,
        mut line: LineMeta,
        state: &mut ParseState,
    ) -> Result<(), ParserError> {
        line.width = match line.op_code {
            Some(op_code) => self.instruction_width(&line, op_code, state.pc()?)?,
            None => self.macro_call_width(&line, state)?,
        };

        let Some(name) = &state.macro_name else {
            line.address = state.pc()?;
            state.advance(line.width)?;
            trace!("@{:<03} placed at {:#06x}", line.line_no, line.address);
            self.lines.push(line);
            return Ok(());
        };

        if matches!(line.inst.as_deref(), Some("DB" | "DW" | "DS")) {
            return Err(ParserError::DefineInMacro);
        }
        // Implementation limitation, not part of the spec
        if state.in_if {
            return Err(ParserError::IfAndMacroMix);
        }
        line.address = state.macro_offset;
        state.macro_offset = u16::try_from(line.width)
            .ok()
            .and_then(|width| state.macro_offset.checked_add(width))
            .ok_or(ParserError::AddressOverflow)?;
        let m = self.macros.entry(name.clone()).or_default();
        m.width += line.width;
        m.lines.push(line);
        Ok(())
    }

    fn instruction_width(
        &self,
        line: &LineMeta,
        op_code: u16,
        pc: u16,
    ) -> Result<usize, ParserError> {
        let meta = I8080_OP_META[op_code as usize];
        if meta.define {
            self.width_of_data_storage(meta.op, &line.args_list, pc)
        } else {
            Ok(meta.width())
        }
    }

    fn macro_call_width(&self, line: &LineMeta, state: &ParseState) -> Result<usize, ParserError> {
        let name = line.inst.as_deref().unwrap_or_default();
        if !line.args_list.is_empty() {
            return Err(ParserError::WrongNumberOfArgs(0, line.args_list.len()));
        }
        if state.macro_name.as_deref() == Some(name) {
            return Err(ParserError::RecursiveMacro);
        }
        self.macros
            .get(name)
            .map(|m| m.width)
            .ok_or_else(|| ParserError::MacroUseBeforeCreation(name.to_string()))
    }

    fn width_of_data_storage(
        &self,
        inst: &str,
        args: &[String],
        pc: u16,
    ) -> Result<usize, ParserError> {
        match inst {
            "DB" => args
                .iter()
                .map(|arg| {
                    let (bytes, flags) = parse_expression(arg, pc, &self.labels)?;
                    Ok(if flags.string { bytes.len() } else { 1 })
                })
                .sum(),
            "DW" => Ok(args.len() * 2),
            "DS" => {
                let arg = args.first().ok_or(ParserError::NoArgsForVariadic)?;
                match parse_expression_u16(arg, pc, &self.labels)? {
                    (_, flags) if flags.string => Err(ParserError::InvalidArgument(
                        inst.to_string(),
                        arg.to_string(),
                    )),
                    (width, _) => Ok(width.into()),
                }
            }
            _ => Err(ParserError::UnknownDefine(inst.to_string())),
        }
    }

    /// Second pass, generates the bytes for each line into a single image beginning at the load
    /// address, gaps left by `ORG` are zero-filled
    fn generate_prog(&mut self) -> Result<Vec<u8>, CodeGenError> {
        let load_address = usize::from(self.args.load_at);
        let mut bytes = vec![0; (self.end_address as usize).saturating_sub(load_address)];
        for line in &self.lines {
            self.current_line = Some((line.line_no, line.raw_line.clone()));
            let line_bytes = self.gen_for_line(line, line.address)?;
            let start = usize::from(line.address) - load_address;
            bytes
                .get_mut(start..start + line_bytes.len())
                .ok_or(ParserError::AddressOverflow)?
                .copy_from_slice(&line_bytes);
        }
        self.current_line = None;
        if self.args.hlt {
            bytes.push(0x76);
        }
        Ok(bytes)
    }

    fn gen_for_line(&self, line: &LineMeta, address: u16) -> Result<Vec<u8>, CodeGenError> {
        let bytes = match line.op_code {
            Some(_) => self.gen_for_instruction(line, address)?,
            None => self.gen_for_macro_call(line, address)?,
        };
        if bytes.len() != line.width {
            return Err(CodeGenError::UnexpectedLength(line.width, bytes.len()));
        }
        trace!("@{:<03} {:#06x}: {:02x?}", line.line_no, address, bytes);
        Ok(bytes)
    }

    /// Macro bodies are generated per call as they may reference the program counter (`$`)
    fn gen_for_macro_call(&self, line: &LineMeta, address: u16) -> Result<Vec<u8>, CodeGenError> {
        let name = line.inst.as_deref().unwrap_or_default();
        let m = self
            .macros
            .get(name)
            .ok_or_else(|| ParserError::MacroUseBeforeCreation(name.to_string()))?;
        let mut bytes = Vec::with_capacity(m.width);
        for macro_line in &m.lines {
            bytes.extend(self.gen_for_line(macro_line, address.wrapping_add(macro_line.address))?);
        }
        Ok(bytes)
    }

    fn gen_for_instruction(&self, line: &LineMeta, address: u16) -> Result<Vec<u8>, CodeGenError> {
        let inst = line.inst.as_deref().unwrap_or_default();
        let exprs = line
            .args_list
            .iter()
            .map(|arg| parse_expression(arg, address, &self.labels))
            .collect::<Result<Vec<ExprOutput>, _>>()?;
        let mut args = line.args_list.iter().zip(&exprs);

        match inst {
            "DB" => {
                let mut bytes = vec![];
                for (arg, (expr, flags)) in args {
                    if flags.string {
                        bytes.extend(expr);
                    } else {
                        bytes.push(expr_value(inst, arg, expr)? as u8);
                    }
                }
                Ok(bytes)
            }
            "DW" => {
                let mut bytes = vec![];
                for (arg, (expr, _)) in args {
                    bytes.extend(expr_value(inst, arg, expr)?.to_le_bytes());
                }
                Ok(bytes)
            }
            "DS" => {
                let (arg, (expr, _)) = args.next().ok_or(ParserError::NoArgsForVariadic)?;
                Ok(vec![0; expr_value(inst, arg, expr)?.into()])
            }
            _ => self.gen_for_op(inst, &line.args_list, &exprs),
        }
    }

    fn gen_for_op(
        &self,
        inst: &str,
        args: &[String],
        exprs: &[ExprOutput],
    ) -> Result<Vec<u8>, CodeGenError> {
        // SP and PSW are only valid as the first argument
        if exprs.iter().skip(1).any(|(_, flags)| flags.sp || flags.psw) {
            return Err(OpParseError::InvalidRegister.into());
        }
        let first_flags = exprs.first().map(|(_, flags)| *flags).unwrap_or_default();
        let arg = |idx: usize| {
            exprs
                .get(idx)
                .map_or(0, |(expr, _)| util::vec_u8_to_u16(expr))
        };

        let code = find_op_code::from_args_and_sp_psw(
            inst,
            arg(0),
            arg(1),
            first_flags.sp,
            first_flags.psw,
        )?;
        let meta = I8080_OP_META[code];

        let mut bytes = vec![code as u8];
        if meta.argb || meta.argw {
            // The immediate is always the final argument
            let (arg, (expr, _)) = args
                .iter()
                .zip(exprs)
                .next_back()
                .ok_or(ParserError::WrongNumberOfArgs(meta.asm_arg_count, 0))?;
            let value = expr_value(inst, arg, expr)?;
            if meta.argb {
                bytes.push(value as u8);
            } else {
                bytes.extend(value.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}

/// Check an instruction has a label if required and the correct number of arguments
fn validate_op(line: &LineMeta, op_code: u16) -> Result<(), ParserError> {
    let meta = I8080_OP_META[op_code as usize];
    if meta.labelled && line.label.is_none() {
        return Err(ParserError::OperationRequiresLabel(meta.op.to_string()));
    }
    if meta.define {
        if line.args_list.is_empty() {
            return Err(ParserError::NoArgsForVariadic);
        }
    } else if meta.asm_arg_count != line.args_list.len() {
        return Err(ParserError::WrongNumberOfArgs(
            meta.asm_arg_count,
            line.args_list.len(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    impl AssembleArgs {
        fn new() -> Self {
            Self {
                input: PathBuf::new(),
                output: PathBuf::new(),
                load_at: 0,
                no_register_definitions: false,
                hlt: false,
            }
        }
    }

    #[test]
    fn width_of_vararg_db() {
        let ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string()];
        let width = ass
            .width_of_data_storage("DB", &args, 0)
            .expect("should parse db arg");
        assert_eq!(width, 1, "a byte of any value should be width 1");

        let args = vec!["13".to_string(), "12".to_string()];
        let width = ass
            .width_of_data_storage("DB", &args, 0)
            .expect("should parse db arg");
        assert_eq!(width, 2, "a list of bytes should return the length");

        let args = vec!["'this-string'".to_string()];
        let width = ass
            .width_of_data_storage("DB", &args, 0)
            .expect("should parse db arg");
        assert_eq!(width, 11, "'this-string' is eleven chars");
    }

    #[test]
    fn width_of_vararg_dw() {
        let ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string(), "fish".to_string()];
        let width = ass
            .width_of_data_storage("DW", &args, 0)
            .expect("should parse db arg");
        assert_eq!(width, 4, "double the numbers of args");
    }

    #[test]
    fn width_of_vararg_ds() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let args = vec!["13".to_string()];
        let width = ass
            .width_of_data_storage("DS", &args, 0)
            .expect("should parse db arg");
        assert_eq!(width, 13, "the result of the expression");

        let args = vec!["'hello'".to_string()];
        let width = ass.width_of_data_storage("DS", &args, 0);
        assert!(width.is_err(), "should parse db arg");
        let e = width.unwrap_err();
        assert!(matches!(e, ParserError::InvalidArgument(_, _)));
        if let ParserError::InvalidArgument(_, s) = e {
            assert_eq!(s, "'hello'", "erroring arg should be 'hello'");
        }

        ass.labels.insert("_ident".to_string(), Label::new_addr(2));
        let args = vec!["_ident".to_string()];
        let width = ass
            .width_of_data_storage("DS", &args, 0)
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
            line_meta_for_parse("SHLD", 0x22, vec!["0x1234"], None),
            line_meta_for_parse("DS", 0x102, vec!["_l1"], None),
            line_meta_for_parse("ADI", 0xc6, vec!["0x22"], None),
        ];
        let len = raw_lines.len();

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.end_address, 7);

        let resolved_lines = &ass.lines;
        assert_eq!(resolved_lines.len(), len, "no meta instructions");

        let l0 = resolved_lines.first().unwrap();
        assert_eq!(l0.width, 1, "MOV A, B is one byte");
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.width, 1, "DCR B is one byte");
        assert_eq!(l1.address, 1, "DCR B address");

        let l2 = resolved_lines.get(2).unwrap();
        assert_eq!(l2.width, 3, "SHLD takes a u16");
        assert_eq!(l2.address, 2, "SHLD address");

        let l3 = resolved_lines.get(3).unwrap();
        assert_eq!(l3.width, 0, "_l1 refers to address 0");
        assert_eq!(l3.address, 5, "SHLD address");

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
            line_meta_for_parse("SHLD", 0x22, vec!["0x0000"], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
            line_meta_for_parse("IF", 0x107, vec!["0"], None),
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDIF", 0x108, vec![], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.end_address, 5);

        let resolved_lines = &ass.lines;

        assert_eq!(resolved_lines.len(), 3, "meta not included");

        let l0 = resolved_lines.first().unwrap();
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 1, "IF and ENDIF shouldn't affect address");
        assert_eq!(
            l1.args_list,
            vec!["0x0000".to_string()],
            "SHLD with 0x0000 included"
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
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
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
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.end_address, 2);

        let resolved_lines = &ass.lines;

        assert_eq!(resolved_lines.len(), 2, "meta not included");

        let l0 = resolved_lines.first().unwrap();
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
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            LineMeta {
                inst: Some("_m1".to_string()),
                ..Default::default()
            },
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
        ];

        ass.parse(raw_lines).expect("should parse");
        assert_eq!(ass.end_address, 8);

        let resolved_lines = &ass.lines;

        assert_eq!(resolved_lines.len(), 2, "macro not included");

        let l0 = resolved_lines.first().unwrap();
        assert_eq!(l0.address, 0, "should be first instruction");
        assert_eq!(l0.width, 5, "should replace with macro width");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 5, "SHLD should be after macro width");
    }

    #[test]
    fn load_address_with_macro() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
            line_meta_for_parse("ENDM", 0x10a, vec![], None),
            LineMeta {
                inst: Some("_m1".to_string()),
                ..Default::default()
            },
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
        ];

        ass.parse_at(raw_lines, 200).expect("should parse");
        assert_eq!(ass.end_address, 208);

        let resolved_lines = &ass.lines;

        assert_eq!(resolved_lines.len(), 2, "macro not included");

        let l0 = resolved_lines.first().unwrap();
        assert_eq!(l0.address, 200, "should be first instruction");
        assert_eq!(l0.width, 5, "should replace with macro width");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.address, 205, "SHLD should be after macro width");
    }

    #[test]
    fn parse_nested_macro_fails() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m1")),
            line_meta_for_parse("MACRO", 0x109, vec![], Some("_m2")),
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
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
        assert!(matches!(e, ParserError::OrgInMacro));
    }

    #[test]
    fn parse_org_sets_the_address() {
        let mut ass = Assembler::new(AssembleArgs::new());

        let raw_lines = vec![
            line_meta_for_parse("MOV", 0x40, vec!["A", "B"], None),
            line_meta_for_parse("ORG", 0x105, vec!["0X1000"], None),
            line_meta_for_parse("SHLD", 0x22, vec!["0x1111"], None),
        ];

        ass.parse(raw_lines).expect("should parse");

        let resolved_lines = &ass.lines;
        assert_eq!(resolved_lines.len(), 2, "no meta instructions");

        let l0 = resolved_lines.first().unwrap();
        assert_eq!(l0.width, 1, "MOV A, B is one byte");
        assert_eq!(l0.address, 0, "MOV A, B address");

        let l1 = resolved_lines.get(1).unwrap();
        assert_eq!(l1.width, 3, "SHLD takes a u16");
        assert_eq!(l1.address, 0x1000, "SHLD address");
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
            assert_eq!(val.value, 0, "previous definition is 0");
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
            assert_eq!(val.value, 200, "previous definition is 200");
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

        let label = ass.labels.get("label").expect("'label' should exist");
        assert_eq!(label.kind, LabelKind::Set, "label is from SET");
        assert_eq!(label.value, 201, "label is updated value");
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

        assert!(ass.lines.is_empty(), "no generate-able lines");

        let eq200 = ass.labels.get("EQ200").expect("'EQ200' should exist");
        assert_eq!(eq200.kind, LabelKind::Equ, "EQ200 is from EQU");
        assert_eq!(eq200.value, 200, "EQ200 is updated value");

        let set20x = ass.labels.get("SET20X").expect("'SET20X' should exist");
        assert_eq!(set20x.kind, LabelKind::Set, "SET20X is from SET");
        assert_eq!(set20x.value, 200, "SET20X is updated value");
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
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0x4a]);
    }

    #[test]
    fn gen_for_instruction_argb() {
        // The line will be loaded with the first MVI (0x06) but it should then resolve to
        // the true OP after expression parsing (0x2e)
        let line = line_meta_for_gen("MVI", 0x06, vec!["5", "0X12"], None, 0, 2);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0x2e, 0x12]);
    }

    #[test]
    fn gen_for_instruction_argw() {
        let line = line_meta_for_gen("JMP", 0xc3, vec!["0X1234"], None, 0, 3);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0xc3, 0x34, 0x12]);
    }

    #[test]
    fn gen_for_instruction_argb_with_sp() {
        let line = line_meta_for_gen("IN", 0xdb, vec!["$ + 2"], None, 0x212, 2);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass
            .gen_for_instruction(&line, 0x212)
            .expect("should generate");
        assert_eq!(bytes, vec![0xdb, 0x14]);
    }

    #[test]
    fn gen_for_instruction_db() {
        let line = line_meta_for_gen("DB", 0x100, vec!["0X10", "'ABAB'", "$"], None, 0, 4);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0x10, b'A', b'B', b'A', b'B', 0x00]);
    }

    #[test]
    fn gen_for_instruction_dw() {
        let line = line_meta_for_gen("DW", 0x101, vec!["0X1234", "0X10", "'ABPP'"], None, 0, 6);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0x34, 0x12, 0x10, 0x00, b'A', b'B']);
    }

    #[test]
    fn gen_for_instruction_ds() {
        let line = line_meta_for_gen("DS", 0x102, vec!["10"], None, 0, 10);
        let ass = Assembler::new(AssembleArgs::new());
        let bytes = ass.gen_for_instruction(&line, 0).expect("should generate");
        assert_eq!(bytes, vec![0x00; 10]);
    }

    fn macro_no_pc() -> Macro {
        Macro {
            lines: vec![
                line_meta_for_gen("MOV", 0x40, vec!["0", "0"], None, 0x00, 1),
                line_meta_for_gen("MOV", 0x40, vec!["0", "1"], None, 0x01, 1),
                line_meta_for_gen("SHLD", 0x22, vec!["0X1111"], None, 0x02, 3),
            ],
            width: 5,
        }
    }

    fn macro_with_pc() -> Macro {
        Macro {
            lines: vec![
                line_meta_for_gen("MOV", 0x40, vec!["0", "0"], None, 0x00, 1),
                line_meta_for_gen("MOV", 0x40, vec!["0", "1"], None, 0x01, 1),
                line_meta_for_gen("SHLD", 0x22, vec!["$"], None, 0x02, 3),
            ],
            width: 5,
        }
    }

    fn assembler_with_macro(name: &str, m: Macro) -> Assembler {
        let mut ass = Assembler::new(AssembleArgs::new());
        ass.macros.insert(name.to_string(), m);
        ass
    }

    fn macro_call(name: &str, address: u16, width: usize) -> LineMeta {
        LineMeta {
            inst: Some(name.to_string()),
            address,
            width,
            ..Default::default()
        }
    }

    #[test]
    fn gen_macro_no_pc() {
        let ass = assembler_with_macro("_m1", macro_no_pc());
        let bytes = ass
            .gen_for_line(&macro_call("_m1", 0x10, 5), 0x10)
            .expect("macro should compile");
        assert_eq!(bytes, vec![0x40, 0x41, 0x22, 0x11, 0x11]);
    }

    #[test]
    fn gen_macro_with_pc() {
        let ass = assembler_with_macro("_m1", macro_with_pc());
        let bytes = ass
            .gen_for_line(&macro_call("_m1", 0x10, 5), 0x10)
            .expect("macro should compile");
        assert_eq!(
            bytes,
            vec![0x40, 0x41, 0x22, 0x12, 0x00],
            "$ is relative to the call"
        );
    }

    #[test]
    fn gen_prog_with_macro_no_pc() {
        let mut ass = Assembler::new(AssembleArgs::new());

        ass.macros.insert("_m1".to_string(), macro_no_pc());

        ass.labels = register_definitions();

        let lines = vec![
            line_meta_for_gen("MOV", 0x40, vec!["B", "C"], None, 0x00, 1),
            macro_call("_m1", 0x01, 5),
            line_meta_for_gen("SHLD", 0x22, vec!["$"], None, 0x06, 3),
        ];
        ass.lines = lines;
        ass.end_address = 9;

        let bytes = ass.generate_prog().expect("should compile");
        assert_eq!(
            bytes,
            vec![0x41, 0x40, 0x41, 0x22, 0x11, 0x11, 0x22, 0x06, 0x00]
        );
    }

    #[test]
    fn gen_prog_with_macro_with_pc() {
        let mut ass = Assembler::new(AssembleArgs::new());

        ass.macros.insert("_m1".to_string(), macro_with_pc());

        ass.labels = register_definitions();

        let lines = vec![
            line_meta_for_gen("MOV", 0x40, vec!["B", "C"], None, 0x00, 1),
            macro_call("_m1", 0x01, 5),
            line_meta_for_gen("SHLD", 0x22, vec!["$"], None, 0x06, 3),
        ];
        ass.lines = lines;
        ass.end_address = 9;

        let bytes = ass.generate_prog().expect("should compile");
        assert_eq!(
            bytes,
            vec![
                0x41, 0x40, 0x41, 0x22,
                // The SHLD in the macro is now MOV, MOV, MOV, SHLD, so 0x03
                0x03, 0x00, 0x22, 0x06, 0x00
            ]
        );
    }

    fn parse_source(load_at: u16, source: &str) -> Result<Vec<u8>, AssemblerError> {
        let mut ass = Assembler::new(AssembleArgs {
            load_at,
            ..AssembleArgs::new()
        });
        ass.labels = register_definitions();
        let lines = source
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                tokenizer::tokenize(line).unwrap().map(|mut meta| {
                    meta.line_no = idx + 1;
                    meta
                })
            })
            .collect();
        ass.parse_at(lines, load_at)?;
        Ok(ass.generate_prog()?)
    }

    fn assemble_ok(source: &str) -> Vec<u8> {
        parse_source(0, source).expect("should assemble")
    }

    fn assemble_err(source: &str) -> AssemblerError {
        parse_source(0, source).expect_err("should fail")
    }

    #[test]
    fn label_only_line() {
        assert_eq!(assemble_ok("_loop:\nJMP _loop"), vec![0xc3, 0x00, 0x00]);
    }

    #[test]
    fn nested_macro_calls() {
        let bytes = assemble_ok(
            "_a: MACRO\n NOP\n LXI H, $\n ENDM\n_b: MACRO\n MOV A, B\n _a\n ENDM\n NOP\n _b",
        );
        assert_eq!(bytes, vec![0x00, 0x78, 0x00, 0x21, 0x03, 0x00]);
    }

    #[test]
    fn load_address_offsets_output_and_labels() {
        let bytes = parse_source(0x100, "_l: NOP\n JMP _l").expect("should assemble");
        assert_eq!(bytes, vec![0x00, 0xc3, 0x00, 0x01], "no leading padding");

        let bytes =
            parse_source(0x100, "_m: MACRO\n JMP $\n ENDM\n NOP\n _m").expect("should assemble");
        assert_eq!(
            bytes,
            vec![0x00, 0xc3, 0x01, 0x01],
            "$ in macro uses load address once"
        );
    }

    #[test]
    fn ds_wider_than_a_byte() {
        assert_eq!(assemble_ok("DS 300").len(), 300);
    }

    #[test]
    fn register_pair_ops() {
        assert_eq!(
            assemble_ok("LDAX D\nSTAX B\nDAD SP\nPUSH PSW\nRET\nRLC"),
            vec![0x1a, 0x02, 0x39, 0xf5, 0xc9, 0x07]
        );
    }

    #[test]
    fn malformed_input_errors_without_panicking() {
        for source in [
            "IF\nENDIF",
            "MACRO\nENDM",
            "EQU 5",
            "MVI A, 1 +",
            "MVI A, ''",
            "LXI B, SP",
            "MVI A, 1 / 0",
            "_x: EQU 1\n_x: SET 2",
            "_x:\n_x:",
            "ORG 0xffff\nLXI B, 0",
            "_unknown",
            "END_OF_LINE: _m",
        ] {
            let e = assemble_err(source);
            assert!(
                matches!(e, AssemblerError::Parser(_) | AssemblerError::CodeGen(_)),
                "{:?}",
                e
            );
        }
    }

    #[test]
    fn org_before_load_address() {
        let e = parse_source(0x100, "ORG 0x10").expect_err("should fail");
        assert!(matches!(
            e,
            AssemblerError::Parser(ParserError::OrgBeforeLoadAddress(0x10, 0x100))
        ));
    }

    #[test]
    fn program_may_end_at_top_of_memory() {
        let bytes = parse_source(0xfffe, "LXI H, 0").map(|b| b.len());
        assert!(bytes.is_err(), "3 bytes from 0xfffe overflows");
        let bytes = parse_source(0xfffd, "LXI H, 0").expect("fits exactly");
        assert_eq!(bytes.len(), 3);
    }

    #[test]
    fn register_names_free_without_definitions() {
        let mut ass = Assembler::new(AssembleArgs::new());
        let lines = ["A: EQU 0x12", "ADI A"]
            .into_iter()
            .filter_map(|line| tokenizer::tokenize(line).unwrap())
            .collect();
        ass.parse(lines).expect("A is free to define");
        assert_eq!(ass.generate_prog().unwrap(), vec![0xc6, 0x12]);

        let e = assemble_err("A: EQU 0x12");
        assert!(matches!(
            e,
            AssemblerError::Parser(ParserError::LabelAlreadyDefined(_, _))
        ));
    }
}
