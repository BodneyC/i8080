use std::collections::HashMap;

use crate::assembler::label::Label;
use crate::util;

use super::{
    errors::ExpressionError,
    lexer::{ExprFlags, Lexer},
    rpn, shunting_yard,
    token::Token,
};

pub type ExprOutput = (Vec<u8>, ExprFlags);

pub fn parse_expression_u16<S: Into<String>>(
    exp: S,
    addr: u16,
    labels: &HashMap<String, Label>,
) -> Result<(u16, ExprFlags), ExpressionError> {
    parse_expression(exp, addr, labels).and_then(|(v, flags)| Ok((util::vec_u8_to_u16(&v), flags)))
}

pub fn parse_expression<S: Into<String>>(
    exp: S,
    addr: u16,
    labels: &HashMap<String, Label>,
) -> Result<ExprOutput, ExpressionError> {
    let exp = exp.into();
    let mut lexer = Lexer::new();

    let (tokens, flags) = lexer.lex(exp.as_str(), addr, labels).unwrap();

    let out: Vec<u8> = match tokens.len() {
        0 => vec![],
        1 => match &tokens[0] {
            Token::Number(val) => util::u16_to_vec_u8(*val),
            Token::String(s) => s.as_bytes().to_vec(),
            Token::MetaIdentifier(_) => vec![],
            tok => return Err(ExpressionError::NotANumber(tok.clone())),
        },
        _ => {
            for tok in tokens.iter() {
                if let Token::MetaIdentifier(val) = tok {
                    return Err(ExpressionError::MetaUsedInCalculation(val.to_string()));
                }
            }
            let output_queue: Vec<Token> = shunting_yard::transform(tokens)?;
            let val: u16 = rpn::calculate(&output_queue)?;
            util::u16_to_vec_u8(val)
        }
    };

    Ok((out, flags))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_and_vec(line: &str, vec: Vec<u8>) -> ExprFlags {
        let r = parse_expression(line, 0, &HashMap::new());
        assert!(r.is_ok(), "{} is invalid: {}", line, r.unwrap_err());
        let (v, flags) = r.unwrap();
        assert_eq!(v, vec);
        flags
    }

    #[test]
    fn simple_sum() {
        let flags = is_valid_and_vec("2 + 3", vec![0x05, 0x00]);
        assert_eq!(flags, ExprFlags::new());
    }

    #[test]
    fn unary_minus() {
        let flags = is_valid_and_vec("NEG 3", vec![0xfd, 0xff]);
        assert_eq!(flags, ExprFlags::new());
        let flags = is_valid_and_vec("NOT NEG 3", vec![0x02, 0x00]);
        assert_eq!(flags, ExprFlags::new());
        let flags = is_valid_and_vec("NOT NEG (1 + 2)", vec![0x02, 0x00]);
        assert_eq!(flags, ExprFlags::new());
    }

    #[test]
    fn simple_sum_with_pc() {
        let r = parse_expression("2 + $", 3, &HashMap::new());
        assert!(r.is_ok(), "expr is invalid: {}", r.unwrap_err());
        let (v, flags) = r.unwrap();
        assert_eq!(v, vec![0x05, 0x00]);
        let mut _flags = ExprFlags::new();
        _flags.pc = true;
        assert_eq!(flags, _flags);
    }

    #[test]
    fn sums_with_brackets() {
        is_valid_and_vec("2 * (4 - 2)", vec![0x04, 0x00]);
        is_valid_and_vec("(4 - 2) * 2", vec![0x04, 0x00]);
        is_valid_and_vec("(4 - 2) * (4 - 2)", vec![0x04, 0x00]);
    }

    #[test]
    fn psw_and_sp() {
        let flags = is_valid_and_vec("SP", vec![]);
        let mut _flags = ExprFlags::new();
        _flags.sp = true;
        assert_eq!(flags, _flags);

        let flags = is_valid_and_vec("PSW", vec![]);
        let mut _flags = ExprFlags::new();
        _flags.psw = true;
        assert_eq!(flags, _flags);

        let r = parse_expression("PSW + 2", 0, &HashMap::new());
        assert!(r.is_err(), "'PSW + 2' shouldn't be valid: {:?}", r.unwrap());
        let e = r.unwrap_err();
        assert!(matches!(e, ExpressionError::MetaUsedInCalculation(_)));
        if let ExpressionError::MetaUsedInCalculation(val) = e {
            assert_eq!(val, "PSW");
        }
    }

    #[test]
    fn not_as_a_unary() {
        is_valid_and_vec("NOT 0XF12F", vec![0xd0, 0x0e]);
        is_valid_and_vec("NOT NOT 0XF12F", vec![0x2f, 0xf1]);
        is_valid_and_vec("NOT (0XF12D + 2)", vec![0xd0, 0x0e]);
        is_valid_and_vec("12 XOR NOT (0XF12D + 2)", vec![0xdc, 0x0e]);
    }

    #[test]
    fn all_together() {
        let mut labels = HashMap::new();
        labels.insert("_ident".to_string(), Label::new_addr(Some(0xf12d)));

        let r = parse_expression("12 XOR NOT (_ident + 2)", 0, &labels);
        assert!(r.is_ok(), "expr is invalid: {}", r.unwrap_err());
        let (v, _) = r.unwrap();
        assert_eq!(v, vec![0xdc, 0x0e]);
    }
}
