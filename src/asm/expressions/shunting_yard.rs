use super::{
    errors::ExpressionError,
    token::{Assoc, Token},
};

pub fn transform(input_queue: Vec<Token>) -> Result<Vec<Token>, ExpressionError> {
    let mut output_queue: Vec<Token> = vec![];
    let mut stack: Vec<Token> = vec![];
    for tok in input_queue {
        match tok {
            Token::Number(_) => output_queue.push(tok),
            Token::Unary(_) | Token::LParen => stack.push(tok),
            Token::Operator(_, o1_assoc, o1_precedence) => {
                while let Some(top) = stack.last() {
                    let pop = match *top {
                        Token::Operator(_, _, o2_precedence) => match o1_assoc {
                            Assoc::Left => o1_precedence <= o2_precedence,
                            Assoc::Right => o1_precedence < o2_precedence,
                        },
                        Token::Unary(_) => true,
                        _ => false,
                    };
                    if !pop {
                        break;
                    }
                    output_queue.extend(stack.pop());
                }
                stack.push(tok);
            }
            Token::RParen => loop {
                match stack.pop() {
                    Some(Token::LParen) => break,
                    Some(op) => output_queue.push(op),
                    None => return Err(ExpressionError::UnmatchedParens),
                }
            },
            Token::String(_) | Token::MetaIdentifier(_) => {
                return Err(ExpressionError::NotANumber(tok))
            }
        }
    }

    while let Some(op) = stack.pop() {
        match op {
            Token::LParen | Token::RParen => return Err(ExpressionError::UnmatchedParens),
            _ => output_queue.push(op),
        }
    }

    Ok(output_queue)
}
