use crate::ast::Expression;
use crate::lexer::{self, token_to_string};
use crate::lexer::TokenType;
use crate::lexer::TokenType::*;

pub fn eval_expression(iter: &mut core::slice::Iter<TokenType>, line: i64) -> Result<Expression, (String, i64)> {
	match iter.next() {
		Some(IntLiteral(x)) => {
			let mut expr = String::new();
			expr.push_str(x);

			while let Some(x) = iter.next() {
				match (x) {
					Operator(';') | Operator(',') | Newline => break,

					IntLiteral(num) => expr.push_str(num),
					Operator(op) => expr.push(*op),
					err => return Err((format!("expected int literal or operator in numerical expression, got {} instead", lexer::token_to_string(err)), line))
				}
			}

			Ok(Expression::NumericalExpression(expr))
		},
		Some(StringLiteral(x)) => Ok(Expression::StringExpression(x.to_owned())),

		Some(err) => return Err((format!("expected either an int literal or string literal in expression evaulation, but got {} instead", token_to_string(err)), line)),
		_ => return Err((String::from("expected either an int literal or string literal in expression evaulation"), line)),
	}
}