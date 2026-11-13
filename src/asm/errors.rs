//! Various error which may be returned by the assembler

use std::{error::Error, fmt, io};

use crate::asm::label::Label;

use super::expressions::errors::ExpressionError;

#[derive(Debug)]
pub enum AssemblerError {
    FileRead(io::Error),
    Parser(ParserError),
    CodeGen(CodeGenError),
    /// Wraps another error with the source line it originated from
    AtLine {
        line_no: usize,
        raw_line: String,
        source: Box<AssemblerError>,
    },
}

impl Error for AssemblerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FileRead(e) => Some(e),
            Self::Parser(e) => Some(e),
            Self::CodeGen(e) => Some(e),
            Self::AtLine { source, .. } => Some(source.as_ref()),
        }
    }
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileRead(e) => write!(f, "{}", e),
            Self::Parser(e) => write!(f, "{}", e),
            Self::CodeGen(e) => write!(f, "{}", e),
            Self::AtLine {
                line_no,
                raw_line,
                source,
            } => write!(f, "{}\n\n  {: <3}| {}", source, line_no, raw_line),
        }
    }
}

impl From<io::Error> for AssemblerError {
    fn from(e: io::Error) -> Self {
        AssemblerError::FileRead(e)
    }
}

impl From<ParserError> for AssemblerError {
    fn from(e: ParserError) -> Self {
        AssemblerError::Parser(e)
    }
}

impl From<CodeGenError> for AssemblerError {
    fn from(e: CodeGenError) -> Self {
        AssemblerError::CodeGen(e)
    }
}

/// The ParserError should only be logged to the user
#[derive(Debug)]
pub enum ParserError {
    ExpressionError(ExpressionError),
    UnknownDefine(String),

    NoArgsForVariadic,
    EmptyArgument,
    WrongNumberOfArgs(usize, usize),
    OperationRequiresLabel(String),

    InvalidArgument(String, String),
    UnterminatedString(String),
    InvalidLabel(String),

    LabelAlreadyDefined(String, Label),
    NoInstructionFound(OpParseError),

    AddressOverflow,
    OrgBeforeLoadAddress(u16, u16),

    OrgInMacro,
    DefineInMacro,
    NotInMacro,
    NestedMacro,
    MacroUseBeforeCreation(String),
    RecursiveMacro,
    NoEndMacro,

    IfAndMacroMix,

    NestedIf,
    NotInIf,
    NoEndIf,
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpressionError(e) => write!(f, "Expression evaluation error: {}", e),
            Self::UnknownDefine(s) => write!(f, "Unknown vararg: {}", s),

            Self::NoArgsForVariadic => write!(f, "No arguments given for variadic instruction"),
            Self::EmptyArgument => write!(f, "Empty argument"),
            Self::WrongNumberOfArgs(req, rec) => write!(
                f,
                "Invalid argument count, required {}, received {}",
                req, rec
            ),
            Self::OperationRequiresLabel(s) => write!(f, "Operation ({}) requires a label", s),

            Self::InvalidArgument(inst, arg) => write!(f, "Invalid arg for {}, {}", inst, arg),
            Self::UnterminatedString(s) => write!(f, "Unterminated string: [{}]", s),
            Self::InvalidLabel(s) => write!(f, "Invalid label: [{}]", s),

            Self::LabelAlreadyDefined(s, l) => {
                write!(f, "Label ({}) already defined as {:?}", s, l)
            }
            Self::NoInstructionFound(e) => write!(f, "{}", e),

            Self::AddressOverflow => write!(f, "Program exceeds the 64KiB address space"),
            Self::OrgBeforeLoadAddress(org, load) => write!(
                f,
                "ORG address ({:#06x}) is before the load address ({:#06x})",
                org, load
            ),

            Self::OrgInMacro => write!(f, "ORG used in macro"),
            Self::DefineInMacro => write!(f, "Cannot define (DB, DW, DS) in macro"),
            Self::NotInMacro => write!(f, "ENDM found before MACRO"),
            Self::NestedMacro => write!(f, "Nested MACRO not permitted"),
            Self::MacroUseBeforeCreation(s) => {
                write!(
                    f,
                    "Unknown instruction or macro used before definition: {}",
                    s
                )
            }
            Self::RecursiveMacro => write!(f, "Use of macro from within macro definition"),
            Self::NoEndMacro => write!(f, "No ENDM found"),

            Self::IfAndMacroMix => write!(f, "This assembler does not support mixing MACRO and IF"),

            Self::NestedIf => write!(f, "Nested IF not permitted"),
            Self::NotInIf => write!(f, "ENDIF found before IF"),
            Self::NoEndIf => write!(f, "No ENDIF found"),
        }
    }
}

impl From<ExpressionError> for ParserError {
    fn from(e: ExpressionError) -> Self {
        ParserError::ExpressionError(e)
    }
}

impl From<OpParseError> for ParserError {
    fn from(e: OpParseError) -> Self {
        ParserError::NoInstructionFound(e)
    }
}

#[derive(Debug)]
pub enum CodeGenError {
    ParserError(ParserError),
    UnexpectedLength(usize, usize),
}

impl Error for CodeGenError {}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodeGenError::ParserError(e) => write!(f, "{}", e),
            CodeGenError::UnexpectedLength(exp, act) => write!(
                f,
                "Byte length generated ({}) differs from expected ({})",
                act, exp,
            ),
        }
    }
}

impl From<ParserError> for CodeGenError {
    fn from(e: ParserError) -> Self {
        CodeGenError::ParserError(e)
    }
}

impl From<ExpressionError> for CodeGenError {
    fn from(e: ExpressionError) -> Self {
        CodeGenError::ParserError(e.into())
    }
}

impl From<OpParseError> for CodeGenError {
    fn from(e: OpParseError) -> Self {
        CodeGenError::ParserError(e.into())
    }
}

#[derive(Debug)]
pub enum OpParseError {
    UnknownRegister,
    InvalidRegister,
    MovAsHalt,
    NoSuchInstruction(String),
}

impl Error for OpParseError {}

impl fmt::Display for OpParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownRegister => write!(f, "No register represented"),
            Self::InvalidRegister => write!(f, "Register invalid for instruction"),
            Self::MovAsHalt => write!(f, "MOV M, M is not valid (0x76 represents HLT)"),
            Self::NoSuchInstruction(inst) => write!(f, "No instruction represented by [{}]", inst),
        }
    }
}

#[derive(Debug)]
pub enum DisassembleError {
    /// Address past the end of the input
    NoRemainingBytes(usize),
    /// Address, mnemonic, and the remaining bytes
    NotEnoughBytes(usize, &'static str, Vec<u8>),
}

impl Error for DisassembleError {}

impl fmt::Display for DisassembleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoRemainingBytes(addr) => write!(f, "@{} No remaining bytes", addr),
            Self::NotEnoughBytes(addr, op, v) => {
                write!(
                    f,
                    "@{} Not enough bytes for instruction [{}]\n  {:?}...",
                    addr, op, v
                )
            }
        }
    }
}
