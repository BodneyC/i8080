use super::{errors::ExpressionError, token::Token};

pub fn calculate(input_queue: &[Token]) -> Result<u16, ExpressionError> {
    trace!("rpn input queue: {:?}", input_queue);

    let mut stack: Vec<u16> = Vec::new();

    for tok in input_queue {
        match tok {
            Token::Number(n) => stack.push(*n),
            Token::Operator(o, _, _) => {
                let (Some(right), Some(left)) = (stack.pop(), stack.pop()) else {
                    return Err(ExpressionError::MissingOperand(*o));
                };
                stack.push(operate(*o, left, right)?);
            }
            Token::Unary(unary) => {
                let arg = stack.pop().ok_or_else(|| {
                    ExpressionError::CalculationError(format!("no argument for {}", unary))
                })?;
                match unary.as_str() {
                    "NOT" => stack.push(!arg),
                    "NEG" => stack.push(arg.wrapping_neg()),
                    _ => return Err(ExpressionError::UnknownUnary(unary.to_string())),
                }
            }
            tok => return Err(ExpressionError::NotANumber(tok.clone())),
        }
    }

    match stack.pop() {
        Some(n) => {
            trace!("rpn result: {:#04x}", n);
            Ok(n)
        }
        None => Err(ExpressionError::CalculationError(
            "empty expression".to_string(),
        )),
    }
}

fn operate(operator: char, left: u16, right: u16) -> Result<u16, ExpressionError> {
    Ok(match operator {
        '+' => left.wrapping_add(right),
        '-' => left.wrapping_sub(right),
        '*' => left.wrapping_mul(right),
        '/' => left
            .checked_div(right)
            .ok_or(ExpressionError::DivisionByZero)?,
        '&' => left & right,
        '^' => left ^ right,
        '|' => left | right,
        _ => return Err(ExpressionError::UnprocessableChar(operator)),
    })
}
