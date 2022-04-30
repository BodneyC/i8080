use std::{fmt, io};

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
    NoInstructionFound,
    TooManyArguments,
    TooFewArguments,
    NoSuchLabel,
    InvalidExpression,
}

impl std::error::Error for ParserError {}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoInstructionFound => write!(f, ""),
            Self::TooManyArguments => write!(f, ""),
            Self::TooFewArguments => write!(f, ""),
            Self::NoSuchLabel => write!(f, ""),
            Self::InvalidExpression => write!(f, ""),
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
