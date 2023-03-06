use crate::lexer;
use crate::lexer::TokenType::{self, *};

use super::process_function_parmaters;

#[derive(Debug)]
pub enum Expression {
	Numerical(String),
	StringLiteral(String),
	Variable(String), /* expression that contains an identifier */
	FunctionCallExpression(String, Vec<Expression>),
}

pub fn expression_to_string(token: &Expression) -> String {
	match token {
		Expression::Numerical(x) => format!("numerical expression '{x}'"),
		Expression::Variable(x) => format!("expression '{x}'"),

		Expression::StringLiteral(x) => format!("string '{x}'"),
		Expression::FunctionCallExpression(name, _) => format!("function call to '{name}'"),
	}
}

pub fn determine_expression(token: &TokenType, iter: &mut core::slice::Iter<TokenType>, line: i64) -> Result<Expression, (String, i64)> {
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

			Ok(Expression::Numerical(expr))
		},
		Identifier(x) => {
			match peekable.peek() {
				Some(Operator('(')) => {
					let arguments = process_function_parmaters(iter, line)?;
					Ok(Expression::FunctionCallExpression(x.to_owned(), arguments))
				},
				_ => Ok(Expression::Variable(x.to_owned())) 
			}
		}
		StringLiteral(x) => Ok(Expression::StringLiteral(x.to_owned())),

		err => return Err((format!("expected either an int literal or string literal in expression evaulation, but got {} instead", lexer::token_to_string(err)), line)),
	}
}