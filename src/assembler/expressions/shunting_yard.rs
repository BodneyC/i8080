use super::{
    errors::ExpressionError,
    token::{self, Token},
};

pub fn transform(input_queue: Vec<Token>) -> Result<Vec<Token>, ExpressionError> {
    let mut output_queue: Vec<Token> = vec![];
    let mut stack: Vec<Token> = vec![];
    for tok in input_queue.iter() {
        match *tok {
            Token::Number(_) => output_queue.push(tok.to_owned()),
            Token::Unary(_) => stack.push(tok.to_owned()),
            Token::Operator(o1, o1_associativity, o1_precedence) => {
                while stack.len() > 0 {
                    match stack.last() {
                        Some(&Token::Operator(_, _, o2_precedence)) => {
                            if (o1_associativity == token::LEFT_ASSOC
                                && o1_precedence <= o2_precedence)
                                || (o1_associativity == token::RIGHT_ASSOC
                                    && o1_precedence < o2_precedence)
                            {
                                output_queue.push(stack.pop().unwrap());
                            } else {
                                break;
                            }
                        }
                        Some(&Token::Unary(_)) => {
                            output_queue.push(stack.pop().unwrap());
                        }
                        _ => break,
                    }
                }
                stack.push(Token::Operator(o1, o1_associativity, o1_precedence));
            }
            Token::LParen => stack.push(Token::LParen),
            Token::RParen => loop {
                match stack.last() {
                    Some(&Token::LParen) => {
                        stack.pop().unwrap();
                        break;
                    }
                    None => {
                        return Err(ExpressionError::UnmatchedParens);
                    }
                    _ => output_queue.push(stack.pop().unwrap()),
                }
            },
            _ => (),
        }
    }

    // Are there any operators left on the stack?
    while stack.len() > 0 {
        // Pop them off and push them to the output_queue
        let op = stack.pop();
        match op {
            Some(Token::LParen) => {
                return Err(ExpressionError::UnmatchedParens);
            }
            Some(Token::RParen) => {
                return Err(ExpressionError::UnmatchedParens);
            }
            _ => output_queue.push(op.unwrap()),
        }
    }

    Ok(output_queue)
}
