use std::collections::HashMap;
use std::isize;
use std::iter;
use std::str;

use crate::asm::label::Label;

use super::errors::ExpressionError;
use super::token::Token;
use super::token::LEFT_ASSOC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprFlags {
    pub sp: bool,
    pub psw: bool,
    pub pc: bool,
    pub string: bool,
}

impl ExprFlags {
    pub fn new() -> Self {
        Self {
            sp: false,
            psw: false,
            pc: false,
            string: false,
        }
    }
}

fn functions(ident: String) -> Option<Token> {
    match ident.as_str() {
        "XOR" => Some(Token::Operator('^', LEFT_ASSOC, 4)),
        "AND" => Some(Token::Operator('&', LEFT_ASSOC, 4)),
        "OR" => Some(Token::Operator('|', LEFT_ASSOC, 4)),
        "NOT" => Some(Token::Unary(ident)),
        "NEG" => Some(Token::Unary(ident)),
        _ => None,
    }
}

pub struct Lexer<'a> {
    iter: iter::Peekable<str::Chars<'a>>,
    address: u16,
}

impl<'a> Lexer<'a> {
    pub fn new() -> Self {
        Self {
            iter: "".chars().peekable(),
            address: 0,
        }
    }

    pub fn lex(
        &mut self,
        raw_str: &'a str,
        address: u16,
        labels: &HashMap<String, Label>,
    ) -> Result<(Vec<Token>, ExprFlags), ExpressionError> {
        self.iter = raw_str.chars().peekable();
        self.address = address;

        let mut flags: ExprFlags = ExprFlags::new();
        let mut tokens: Vec<Token> = Vec::new();

        // Should we skip advancing if a sub method has done it for us?
        while let Some(&c) = self.iter.peek() {
            let consume_next: bool = if c.is_whitespace() {
                true
            } else if c.is_numeric() {
                let (number, radix) = self.consume_number();
                match isize::from_str_radix(&number, radix) {
                    Ok(val) => {
                        tokens.push(Token::Number(val as u16));
                    }
                    Err(e) => return Err(ExpressionError::NumberParseError(e)),
                }
                false
            } else if c.is_alphabetic() || c == '_' {
                let ident = self.consume_identifier();
                if let Some(operator) = functions(ident.to_string()) {
                    tokens.push(operator);
                } else
                // These should come before the label check as someone might use an SP label
                if ident == "PSW" {
                    flags.psw = true;
                    tokens.push(Token::MetaIdentifier(ident));
                } else if ident == "SP" {
                    flags.sp = true;
                    tokens.push(Token::MetaIdentifier(ident));
                } else if let Some(label) = labels.get(&ident) {
                    tokens.push(Token::Number(label.value.unwrap()));
                } else {
                    return Err(ExpressionError::UnknownIdentifier(ident));
                }
                false
            } else {
                match c {
                    '\'' => {
                        let s = self.consume_string()?;
                        if s.len() == 1 {
                            tokens.push(Token::Number(s.chars().nth(0).unwrap() as u16))
                        } else {
                            flags.string = true;
                            tokens.push(Token::String(s));
                        }
                    }
                    '$' => {
                        flags.pc = true;
                        tokens.push(Token::Number(self.address));
                    }
                    '+' | '-' => {
                        tokens.push(Token::Operator(c, LEFT_ASSOC, 2));
                    }
                    '*' | '/' => {
                        tokens.push(Token::Operator(c, LEFT_ASSOC, 3));
                    }
                    '(' => tokens.push(Token::LParen),
                    ')' => tokens.push(Token::RParen),
                    _ => return Err(ExpressionError::UnprocessableChar(c)),
                }
                true
            };

            if consume_next {
                self.iter.next();
            }
        }

        Ok((tokens, flags))
    }

    fn consume_string(&mut self) -> Result<String, ExpressionError> {
        self.iter.next(); // Consume '\''

        let mut escaped = false;
        let mut quote_matched = false;

        let mut s = String::new();

        while let Some(c) = self.iter.next() {
            if escaped {
                escaped = false;
                let escaped_char: char = match c {
                    '\\' => '\\',
                    '\'' => '\'',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    _ => return Err(ExpressionError::UnknownEscape(c)),
                };
                s.push(escaped_char);
            } else {
                if c == '\'' {
                    quote_matched = true;
                    break;
                }
                if c == '\\' {
                    escaped = true;
                } else {
                    s.push(c);
                }
            }
        }

        if quote_matched {
            Ok(s)
        } else {
            Err(ExpressionError::UnmatchedQuote(s))
        }
    }

    fn consume_number(&mut self) -> (String, u32) {
        let mut s = String::new();
        let mut radix: u32 = 10;
        while let Some(&c) = self.iter.peek() {
            if c.is_numeric() {
                s.push(c);
            } else
            // Handle [A-F] for hexadecimals
            if radix == 16 && c >= 'A' && c <= 'F' {
                s.push(c);
            } else if s.len() == 1 && s.chars().nth(0).unwrap() == '0' {
                if c == 'X' {
                    radix = 16;
                } else if c == 'B' {
                    radix = 2;
                } else if c == 'O' {
                    radix = 8;
                } else {
                    break;
                }
            } else {
                break;
            }
            self.iter.next();
        }
        (s, radix)
    }

    // Consumes an identifier until we don't have any other letters available
    fn consume_identifier(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.iter.peek() {
            if c.is_alphabetic() || c == '_' || c.is_numeric() {
                s.push(c);
            } else {
                break;
            }
            self.iter.next();
        }
        s
    }

    #[cfg(test)]
    fn set_input(&mut self, raw_str: &'a str, address: u16) {
        self.iter = raw_str.chars().peekable();
        self.address = address;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers() {
        let mut lexer = Lexer::new();

        let s = "_ident ident __id__ent__ other";
        lexer.set_input(s, 0);

        let out = lexer.consume_identifier();
        assert_eq!(out, "_ident".to_string());

        lexer.iter.next(); // Whitespace

        let out = lexer.consume_identifier();
        assert_eq!(out, "ident".to_string());

        lexer.iter.next(); // Whitespace

        let out = lexer.consume_identifier();
        assert_eq!(out, "__id__ent__".to_string());

        let next = lexer.iter.next(); // Whitespace

        assert!(next.is_some(), "' other' should remain");
        assert_eq!(next.unwrap(), ' ', "' ' is next");

        let s = "_ident8080";
        lexer.set_input(s, 0);

        let out = lexer.consume_identifier();
        assert_eq!(out, "_ident8080".to_string());

        let next = lexer.iter.next();

        assert!(next.is_none(), "numerics consumed too");
    }

    #[test]
    fn strings() {
        let mut lexer = Lexer::new();

        let s = "'hello' 'hello\\ngoodbye' 'split\\'up' 'this\\\\that'";
        lexer.set_input(s, 0);

        let out = lexer.consume_string();
        assert!(out.is_ok(), "'hello' should be a valid string");
        assert_eq!(out.unwrap(), "hello");

        lexer.iter.next(); // Whitespace

        let out = lexer.consume_string();
        assert!(out.is_ok(), "'hello\\ngoodbye' should be a valid string");
        assert_eq!(out.unwrap(), "hello\ngoodbye");

        lexer.iter.next(); // Whitespace

        let out = lexer.consume_string();
        assert!(out.is_ok(), "'split\\'up' should be a valid string");
        assert_eq!(out.unwrap(), "split'up");

        lexer.iter.next(); // Whitespace

        let out = lexer.consume_string();
        assert!(out.is_ok(), "'this\\\\that' should be a valid string");
        assert_eq!(out.unwrap(), "this\\that");
    }

    fn is_number_of_value(t: &Token, exp: u16, mes: &str) {
        assert!(matches!(t, Token::Number(_)));
        if let Token::Number(n) = t {
            assert_eq!(*n, exp, "{}", mes);
        }
    }

    #[test]
    fn numerics() {
        let mut lexer = Lexer::new();

        let out = lexer.lex("0B10 0O10 10 0X10 0XFF", 0, &HashMap::new());

        assert!(out.is_ok(), "lexing failed: {}", out.unwrap_err());
        let (tokens, flags) = out.unwrap();
        assert_eq!(tokens.len(), 5, "should be four tokens");
        assert_eq!(flags, ExprFlags::new());

        is_number_of_value(tokens.get(0).unwrap(), 2, "binary parse");
        is_number_of_value(tokens.get(1).unwrap(), 8, "octal parse");
        is_number_of_value(tokens.get(2).unwrap(), 10, "decimal parse");
        is_number_of_value(tokens.get(3).unwrap(), 16, "hexadecimal parse");
        is_number_of_value(tokens.get(4).unwrap(), 0xff, "hexadecimal parse");
    }

    fn is_op_of_code(t: &Token, exp: char) {
        assert!(matches!(t, Token::Operator(_, _, _)));
        if let Token::Operator(op, _, _) = t {
            assert_eq!(*op, exp);
        }
    }

    #[test]
    fn lex() {
        let label_string = "_ident".to_string();
        let label_val = 3213;

        let mut lexer = Lexer::new();

        let mut labels = HashMap::new();
        labels.insert(label_string.to_string(), Label::new_addr(Some(label_val)));

        let s = format!("'hello' XOR+17 {}", label_string);

        let out = lexer.lex(&s, 0, &labels);
        assert!(out.is_ok(), "lexing failed: {}", out.unwrap_err());
        let (tokens, flags) = out.unwrap();
        assert_eq!(tokens.len(), 5, "should be five tokens");
        let mut _flags = ExprFlags::new();
        _flags.string = true;
        assert_eq!(flags, _flags);

        let t0 = tokens.get(0).unwrap();
        assert!(matches!(t0, Token::String(_)));
        if let Token::String(s) = t0 {
            assert_eq!(s, "hello");
        }

        is_op_of_code(tokens.get(1).unwrap(), '^');
        is_op_of_code(tokens.get(2).unwrap(), '+');
        is_number_of_value(tokens.get(3).unwrap(), 17, "");
        is_number_of_value(tokens.get(4).unwrap(), label_val, "");
    }
}
