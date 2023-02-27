use crate::lexer::{TokenType, token_to_string};
use crate::lexer::TokenType::*;

use std::collections::HashMap;

#[derive(Debug)]
pub enum AstType<'a> {
	/* function name, optional hashmap of paramaters (key being the identifier, value being the datatype) */
	FunctionDefinition(&'a str, Option<HashMap<String, String>>),
	/* variable name, type, and initializer value */
	VariableDefinition(&'a str, &'a str, &'a str),
	/* macro name, arguments */
	MacroCall(&'a str, Vec<&'a TokenType>),
	/* function name, arguments */
	FunctionCall(&'a str, Vec<&'a TokenType>),
	/* this is so that functions know when they end */
	ScopeEnd,
	/* for counting the line number in parser.rs */
	Newline
}

pub fn ast(input: &[TokenType]) -> Result<Vec<AstType>, (String, i64)> {
	let mut ast: Vec<AstType> = Vec::new();
	let mut line: i64 = 1;

	let mut iter = input.iter();
	while let Some(i) = iter.next() {
		match i {
			/* increment line number on newline token */
			Newline => {
				line += 1;
				ast.push(AstType::Newline);
			},
			/* scope end */
			Operator('}') => ast.push(AstType::ScopeEnd),
			/* function definitions */
			Keyword(keyword) if keyword == "fn" => {
				let function_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after function keyword".to_owned(), line))
				};

				ast.push(AstType::FunctionDefinition(function_name, None));
			},
			/* variable declerations */
			Keyword(keyword) if keyword == "let" => {
				/* get variable name */
				let variable_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after let keyword".to_owned(), line))
				};

				/* check if there's a colon after variable name */
				match iter.next() {
					Some(Operator(':')) => (),
					_ => return Err(("expected operator ':' token after the variable name".to_owned(), line))
				};
				
				/* get variable type */
				let variable_type = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after operator ':'".to_owned(), line))
				};

				/* check if there's a = after the type name */
				match iter.next() {
					Some(Operator('=')) => (),
					_ => return Err(("expected operator '=' after type name".to_owned(), line))
				};

				/* get initializer value */
				let intializer_value = match iter.next() {
					Some(IntLiteral(x)) => x,
					Some(StringLiteral(x)) => x,
					_ => return Err(("expected either an int literal or string literal in variable intializer after '=' operator".to_owned(), line))
				};

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name, variable_type, intializer_value));
			}
			/* builtin macro calls */
			Identifier(identifier) => {
				/* check for ( */
				match iter.next() {
					Some(Operator('(')) => (),
					_ => return Err((format!("expected operator '(' after macro '{identifier}'"), line))
				}

				let mut arguments: Vec<&TokenType> = Vec::new();

				#[allow(irrefutable_let_patterns)]
				while let i = iter.next() {
					let token = match i {
						Some(Operator(')')) => break,
						Some(Operator(';')) | None => return Err((format!("expected a closing ) in call to macro {identifier}"), line)),
						Some(x) => x
					};

					match (token) {
						StringLiteral(_) | IntLiteral(_) | Identifier(_) => arguments.push(token),
						Operator(',') | Newline => (),
						err => return Err((format!("unexpected {} in call to macro {identifier}", token_to_string(err)), line))
					}
				}

				if (identifier.ends_with("!")) {
					ast.push(AstType::MacroCall(identifier, arguments));
				}
				else {
					ast.push(AstType::FunctionCall(identifier, arguments));
				}
			}
			_ => (),
		}
	}

	Ok(ast)
}