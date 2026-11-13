#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Assoc {
    Left,
    #[allow(dead_code)] // No right-associative operators exist yet
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// Operator, associativity, precedence
    Operator(char, Assoc, u32),
    String(String),
    Number(u16),
    MetaIdentifier(String),
    LParen,
    RParen,
    Unary(String),
}
