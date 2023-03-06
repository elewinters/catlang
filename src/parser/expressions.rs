use std::fmt::{self, Display};
use crate::lexer::TokenType::{self, *};

use super::process_function_parmaters;

#[derive(Debug)]
pub enum Expression {
	Numerical(String),
	StringLiteral(String),
	Variable(String), /* expression that contains an identifier */
	FunctionCallExpression(String, Vec<Expression>),
}

impl Display for Expression {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Expression::Numerical(x) => write!(f, "numerical expression '{x}'"),
			Expression::Variable(x) => write!(f, "variable '{x}'"),

			Expression::StringLiteral(x) => write!(f, "string literal '{x}'"),
			Expression::FunctionCallExpression(name, _) => write!(f, "function call to '{name}'"),
		}
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
					err => return Err((format!("expected int literal or operator in numerical expression, got {err} instead"), line))
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

		err => return Err((format!("expected either an int literal or string literal in expression evaulation, but got {err} instead"), line)),
	}
}