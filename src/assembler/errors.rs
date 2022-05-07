use std::{fmt, io};

use crate::assembler::label::Label;

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
            Self::FileReadError(e) => write!(f, "failed to open file for read: {:?}", e),
            Self::ParserError(_) => write!(f, "error occured in parsing"),
            Self::CodeGenError(_) => write!(f, "error occured in byte generation"),
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

    NoArgsForVariadic,
    WrongNumberOfArgs(usize, usize),
    OperationRequiresLabel(String),

    InvalidExpression,
    UnterminatedString(String),
    InvalidLabel(String),

    NoSuchLabel,
    LabelAlreadyDefined(String, Label),
    NoSuchMacro(String),
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
            Self::ExpressionError(e) => write!(f, "expression evaluation error: {:?}", e),

            Self::NoArgsForVariadic => write!(f, "no arguments given for variadic instruction"),
            Self::WrongNumberOfArgs(req, rec) => write!(
                f,
                "invalid argument count, required {}, received {}",
                req, rec
            ),
            Self::OperationRequiresLabel(s) => write!(f, "operation ({}) required label", s),

            Self::InvalidExpression => write!(f, ""),
            Self::UnterminatedString(s) => write!(f, "unterminated string: '{}'", s),
            Self::InvalidLabel(s) => write!(f, "invalid label: '{}'", s),

            Self::NoSuchLabel => write!(f, ""),
            Self::LabelAlreadyDefined(s, l) => {
                write!(f, "label ({}) already defined at {:?}", s, l)
            }
            Self::NoSuchMacro(s) => write!(f, "no macro found ({})", s),
            Self::NoInstructionFound(_) => write!(f, ""),

            Self::OrigInMacro => write!(f, "ORIG used in macro"),
            Self::DefineInMacro => write!(f, "cannot define (DB DW DB) in macro"),
            Self::NotInMacro => write!(f, "ENDM found before MACRO"),
            Self::NestedMacro => write!(f, "nested MACRO not permitted"),
            Self::MacroCallInMacroUsesSp => write!(
                f,
                "use of a macro which uses SP ('$') inside another macro is not permitted"
            ),
            Self::MacroUseBeforeCreation => write!(f, "macro used before its definition"),
            Self::RecursiveMacro => write!(f, "use of macro from within macro definition"),
            Self::NoEndMacro => write!(f, "no ENDM found"),

            Self::IfAndMacroMix => write!(f, "this assembler does not support mixing MACRO and IF"),

            Self::NestedIf => write!(f, "nested IF not permitted"),
            Self::NotInIf => write!(f, "ENDIF found before IF"),
            Self::NoEndIf => write!(f, "no ENDIF found"),
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
            CodeGenError::ParserError(e) => write!(f, "expression error: {}", e),
            CodeGenError::UnexpectedLength(exp, act) => write!(
                f,
                "byte length generate ({}) differs from expected ({})",
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
            Self::UnknownRegister => write!(f, "no register represented"),
            Self::InvalidRegister => write!(f, "register invalid for instruction"),
            Self::MovAsHalt => write!(f, "0x76 represents a HALT"),
            Self::NoSuchInstruction(inst) => write!(f, "no instruction represented by '{}'", inst),
        }
    }
}
