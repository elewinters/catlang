use crate::lexer;
use crate::lexer::TokenType::{self, *};

use super::process_function_parmaters;

#[derive(Debug)]
pub enum Expression {
	NumericalExpression(String),
	StringExpression(String),
	Expression(String), /* essentially an "IdentifierExpression" for now */
	FunctionCallExpression(String, Vec<Expression>),
}

pub fn expression_to_string(token: &Expression) -> String {
	match token {
		Expression::NumericalExpression(x) => format!("numerical expression '{x}'"),
		Expression::Expression(x) => format!("expression '{x}'"),

		Expression::StringExpression(x) => format!("string '{x}'"),
		Expression::FunctionCallExpression(name, _) => format!("function call to '{name}'"),
	}
}

pub fn eval_expression(token: &TokenType, iter: &mut core::slice::Iter<TokenType>, line: i64) -> Result<Expression, (String, i64)> {
	let mut peekable = iter.peekable();
	match token {
		IntLiteral(x) => {
			let mut expr = String::new();
			expr.push_str(x);

			for x in iter {
				match (x) {
					Operator(';') | Operator(',') | Operator(')') => break,

					IntLiteral(num) => expr.push_str(num),
					Operator(op) => expr.push(*op),
					err => return Err((format!("expected int literal or operator in numerical expression, got {} instead", lexer::token_to_string(err)), line))
				}
			}

			Ok(Expression::NumericalExpression(expr))
		},
		Identifier(x) => {
			match peekable.peek() {
				Some(Operator('(')) => {
					let arguments = process_function_parmaters(iter, line)?;
					Ok(Expression::FunctionCallExpression(x.to_owned(), arguments))
				},
				_ => Ok(Expression::Expression(x.to_owned())) 
			}
		}
		StringLiteral(x) => Ok(Expression::StringExpression(x.to_owned())),

		err => return Err((format!("expected either an int literal or string literal in expression evaulation, but got {} instead", lexer::token_to_string(err)), line)),
	}
}