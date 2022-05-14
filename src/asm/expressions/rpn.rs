use super::{errors::ExpressionError, token::Token};

pub fn calculate(input_queue: &Vec<Token>) -> Result<u16, ExpressionError> {
    trace!("rpn input queue: {:?}", input_queue);

    let mut stack = Vec::new();

    for tok in input_queue.iter() {
        match tok {
            Token::Number(n) => stack.push(Token::Number(*n)),
            Token::Operator(o, _, _) => {
                let right = stack.pop();
                let left = stack.pop();
                match (left, right) {
                    (Some(Token::Number(n1)), Some(Token::Number(n2))) => {
                        stack.push(Token::Number(operate(*o, n1, n2)))
                    }
                    _ => break,
                }
            }
            Token::Unary(unary) => match unary.as_str() {
                "NOT" => {
                    let arg = stack.pop();

                    if let Some(Token::Number(n)) = arg {
                        stack.push(Token::Number(!n));
                    }
                }
                "NEG" => {
                    let arg = stack.pop();

                    if let Some(Token::Number(n)) = arg {
                        stack.push(Token::Number((0 as u16).wrapping_sub(n)));
                    }
                }
                _ => return Err(ExpressionError::UnknownUnary(unary.to_string())),
            },
            _ => (),
        }
    }

    match stack.pop() {
        Some(Token::Number(n)) => {
            trace!("rpn result: {:#04x}", n);
            Ok(n)
        }
        Some(tok) => Err(ExpressionError::NotANumber(tok)),
        None => Err(ExpressionError::CalculationError("unknown".to_string())),
    }
}

fn operate(operator: char, left: u16, right: u16) -> u16 {
    match operator {
        '+' => left.wrapping_add(right),
        '-' => left.wrapping_sub(right),
        '*' => left.wrapping_mul(right),
        '/' => left.wrapping_div(right),
        '&' => left & right,
        '^' => left ^ right,
        '|' => left | right,
        _ => 0,
    }
}
