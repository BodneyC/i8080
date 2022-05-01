use std::{fmt, io};

use crate::assembler::label::Label;

#[derive(Debug)]
pub enum AssemblerError {
    FileReadError(io::Error),
    ParseError(ParserError),
}

impl std::error::Error for AssemblerError {}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::FileReadError(e) => write!(f, "failed to open file for read: {:?}", e),
            Self::ParseError(_) => write!(f, "error occured in parsing input"),
        }
    }
}

/// The ParserError should only be logged to the user
#[derive(Debug)]
pub enum ParserError {
    NoInstructionFound(OpParseError),
    NoMacroFound(String),
    WrongNumberOfArgs(usize, usize),
    NoSuchLabel,
    InvalidExpression,
    LabelAlreadyDefined(String, Label),
    OperationRequiresLabel(String),
    InvalidSyntax(&'static str),
    NotInMacro,
    NoEndMacro,
    NotInIf,
    NoEndIf,
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoInstructionFound(e) => write!(f, ""),
            Self::NoMacroFound(s) => write!(f, "no macro found ({})", s),
            Self::WrongNumberOfArgs(req, rec) => write!(
                f,
                "invalid argument count, required {}, received {}",
                req, rec
            ),
            Self::NoSuchLabel => write!(f, ""),
            Self::InvalidExpression => write!(f, ""),
            Self::OperationRequiresLabel(s) => write!(f, "operation ({}) required label", s),
            Self::LabelAlreadyDefined(s, l) => {
                write!(f, "label ({}) already defined at {:?}", s, l)
            }
            Self::InvalidSyntax(s) => write!(f, "invalid syntax: {}", s),
            Self::NotInMacro => write!(f, "ENDM found before MACRO"),
            Self::NoEndMacro => write!(f, "no ENDM found"),
            Self::NotInIf => write!(f, "ENDIF found before IF"),
            Self::NoEndIf => write!(f, "no ENDIF found"),
        }
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
            Self::UnknownRegister => write!(f, "no register represented"),
            Self::InvalidRegister => write!(f, "register invalid for instruction"),
            Self::MovAsHalt => write!(f, "0x76 represents a HALT"),
            Self::NoSuchInstruction(inst) => write!(f, "no instruction represented by '{}'", inst),
        }
    }
}
