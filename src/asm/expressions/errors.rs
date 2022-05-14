use std::{fmt, num::ParseIntError};

use super::token::Token;

#[derive(Debug)]
pub enum ExpressionError {
    UnprocessableChar(char),
    UnknownIdentifier(String),
    NumberParseError(ParseIntError),
    UnmatchedQuote(String),
    UnmatchedParens,
    UnknownEscape(char),
    CalculationError(String),
    UnknownUnary(String),
    NotANumber(Token),
    MetaUsedInCalculation(String),
}

impl std::error::Error for ExpressionError {}

impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnprocessableChar(c) => write!(f, "unprocessable char: '{}'", c),
            Self::UnknownIdentifier(ident) => write!(f, "unknown identifier '{}'", ident),
            Self::NumberParseError(e) => write!(f, "{}", e),
            Self::UnmatchedQuote(s) => write!(f, "quote is unmatched [{}]", s),
            Self::UnmatchedParens => write!(f, "parens are unmatched"),
            Self::UnknownEscape(c) => write!(f, "unknown escape char [\\{}]", c),
            Self::CalculationError(s) => write!(f, "error in rpn calculator: {}", s),
            Self::UnknownUnary(s) => write!(f, "unknown function '{}'", s),
            Self::NotANumber(t) => write!(f, "calculation yielded NaN: {:?}", t),
            Self::MetaUsedInCalculation(s) => write!(f, "meta arg used in calculation: {}", s),
        }
    }
}
