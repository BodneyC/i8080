//! Various error which may be returned by the assembler

use std::{fmt, io};

use crate::asm::label::Label;

use super::expressions::errors::ExpressionError;

#[derive(Debug)]
pub enum AssemblerError {
    FileReadError(io::Error),
    ParserError(ParserError),
    CodeGenError(CodeGenError),
}

impl std::error::Error for AssemblerError {}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileReadError(e) => write!(f, "{}", e),
            Self::ParserError(e) => write!(f, "{}", e),
            Self::CodeGenError(e) => write!(f, "{}", e),
        }
    }
}

impl From<ParserError> for AssemblerError {
    fn from(e: ParserError) -> Self {
        AssemblerError::ParserError(e)
    }
}

impl From<CodeGenError> for AssemblerError {
    fn from(e: CodeGenError) -> Self {
        AssemblerError::CodeGenError(e)
    }
}

/// The ParserError should only be logged to the user
#[derive(Debug)]
pub enum ParserError {
    ExpressionError(ExpressionError),
    UnknownDefine(String),

    NoArgsForVariadic,
    WrongNumberOfArgs(usize, usize),
    OperationRequiresLabel(String),

    // InvalidExpression,
    InvalidArgument(String, String),
    UnterminatedString(String),
    InvalidLabel(String),

    LabelAlreadyDefined(String, Label),
    NoInstructionFound(OpParseError),

    OrigInMacro,
    DefineInMacro,
    NotInMacro,
    NestedMacro,
    MacroCallInMacroUsesSp,
    MacroUseBeforeCreation,
    RecursiveMacro,
    NoEndMacro,

    IfAndMacroMix,

    NestedIf,
    NotInIf,
    NoEndIf,
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpressionError(e) => write!(f, "Expression evaluation error: {:?}", e),
            Self::UnknownDefine(s) => write!(f, "Unknown vararg: {}", s),

            Self::NoArgsForVariadic => write!(f, "No arguments given for variadic instruction"),
            Self::WrongNumberOfArgs(req, rec) => write!(
                f,
                "Invalid argument count, required {}, received {}",
                req, rec
            ),
            Self::OperationRequiresLabel(s) => write!(f, "Operation ({}) required label", s),

            // Self::InvalidExpression => write!(f, ""),
            Self::InvalidArgument(inst, arg) => write!(f, "Invalid arg for {}, {}", inst, arg),
            Self::UnterminatedString(s) => write!(f, "Unterminated string: [{}]", s),
            Self::InvalidLabel(s) => write!(f, "Invalid label: [{}]", s),

            Self::LabelAlreadyDefined(s, l) => {
                write!(f, "Label ({}) already defined at {:?}", s, l)
            }
            Self::NoInstructionFound(_) => write!(f, ""),

            Self::OrigInMacro => write!(f, "ORIG used in macro"),
            Self::DefineInMacro => write!(f, "Cannot define (DB DW DB) in macro"),
            Self::NotInMacro => write!(f, "ENDM found before MACRO"),
            Self::NestedMacro => write!(f, "Nested MACRO not permitted"),
            Self::MacroCallInMacroUsesSp => write!(
                f,
                "Use of a macro which uses SP ('$') inside another macro is not permitted"
            ),
            Self::MacroUseBeforeCreation => write!(f, "Macro used before its definition"),
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

#[derive(Debug)]
pub enum CodeGenError {
    ParserError(ParserError),
    UnexpectedLength(usize, usize),
}

impl std::error::Error for CodeGenError {}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodeGenError::ParserError(e) => write!(f, "Expression error: {}", e),
            CodeGenError::UnexpectedLength(exp, act) => write!(
                f,
                "Byte length generate ({}) differs from expected ({})",
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
        CodeGenError::ParserError(ParserError::ExpressionError(e))
    }
}

#[derive(Debug)]
pub enum OpParseError {
    UnknownRegister,
    InvalidRegister,
    MovAsHalt,
    NoSuchInstruction(String),
}

impl std::error::Error for OpParseError {}

impl fmt::Display for OpParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownRegister => write!(f, "No register represented"),
            Self::InvalidRegister => write!(f, "Register invalid for instruction"),
            Self::MovAsHalt => write!(f, "0x76 represents a HALT"),
            Self::NoSuchInstruction(inst) => write!(f, "No instruction represented by [{}]", inst),
        }
    }
}

#[derive(Debug)]
pub enum DisassembleError {
    NoRemainingBytes(usize),                      // addr, width
    NotEnoughBytes(usize, &'static str, Vec<u8>), // addr, inst, ~3 bytes
    UnknownError(usize, Vec<u8>),                 // addr, ~3 bytes
}

impl std::error::Error for DisassembleError {}

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
            Self::UnknownError(addr, v) => {
                write!(f, "@{} Unknown disassembly error\n  {:?}...", addr, v)
            }
        }
    }
}
