#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Operator(char, u32, u32), // Operator, associativity, precedence
    String(String),
    Number(u16),
    MetaIdentifier(String),
    LParen,
    RParen,
    Unary(String),
}

pub const LEFT_ASSOC: u32 = 1;
pub const RIGHT_ASSOC: u32 = 2;
