//! Expression-parser
//!
//! This is a Shunting-yard algorithm loosely based on <https://github.com/simon-whitehead/rust-yard>
//!
//! Several modifications have been made to make is suitable for the 8080:
//!
//! Labels from the assembler are passed into to be resolved within the expressions
//!
//! ## Operations
//!
//! Some operators are given as text, namely the bitwise ones:
//!
//! - `XOR` is a binary operation for bitwise-exclusive-or
//! - `AND` is a binary operation for bitwise-and
//! - `OR` is a binary operation for bitwise-or
//! - `NOT` is a unary operation for bitwise-not
//! - `NEG` is a unary operation for negation
//!
//! `NEG` is not in the original spec for the language, however it is quite useful.
//!
//! ## Numerics
//!
//! Another thing away from the standard spec is the specification of numeric-literals with radices
//! other than ten.
//!
//! If you wish to provide the value 16 as hex, the spec would have you give `10H`, however this is
//! harder to parse and honestly less common these days - this expression parser would prefer `0x10`
//!
//! Similarly (for the value 16):
//!
//! | Radix | By-the-spec | This Parser |
//! | :--   | :--         | :--         |
//! | 2     | 10000B      | 0b10000     |
//! | 8     | 20O         | 0o20        |
//! | 10    | 16          | 16          |
//! | 16    | 10H         | 0x10        |
//!
//! ## Examples
//!
//! ```
//! -2    ; becomes
//! NEG 2
//! ```
//!
//! ```
//! 254 + 12    ; becomes
//! 0xfe + 0o14
//! ```
//!
//! ```
//! 12 ^ (- (5 & 3))     ; becomes
//! 12 XOR NEG (5 AND 3)
//! ```


pub mod errors;
pub mod parser;

mod lexer;
mod rpn;
mod shunting_yard;
mod token;
