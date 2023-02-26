use crate::lexer::TokenType;
use crate::lexer::TokenType::*;

use std::collections::HashMap;

#[derive(Debug)]
pub enum AstType {
	/* function name, optional hashmap of paramaters (key being the identifier, value being the datatype) */
	FunctionDefinition(String, Option<HashMap<String, String>>),
	/* variable name, type, and initializer value */
	VariableDefinition(String, String, String),
	/* macro name, arguments */
	BuiltinMacroCall(String, String)
}

pub fn ast(input: &[TokenType]) -> Result<Vec<AstType>, (String, i64)> {
	let mut ast: Vec<AstType> = Vec::new();
	let mut line: i64 = 1;

	let mut iter = input.iter();
	while let Some(i) = iter.next() {
		match i {
			/* increment line number on newline token */
			Newline => line += 1,
			/* function definitions */
			Keyword(keyword) if keyword == "fn" => {
				let function_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after function keyword".to_owned(), line))
				};

				ast.push(AstType::FunctionDefinition(function_name.to_owned(), None));
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
					Some(Operator(operator)) if *operator == ':' => (),
					_ => return Err(("expected operator ':' token after the variable name".to_owned(), line))
				};
				
				/* get variable type */
				let variable_type = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after operator ':'".to_owned(), line))
				};

				/* check if there's a = after the type name */
				match iter.next() {
					Some(Operator(operator)) if *operator == '=' => (),
					_ => return Err(("expected operator '=' after type name".to_owned(), line))
				};

				/* get initializer value */
				let intializer_value = match iter.next() {
					Some(IntLiteral(x)) => x,
					Some(StringLiteral(x)) => x,
					_ => return Err(("expected either an int literal or string literal in variable intializer after '=' operator".to_owned(), line))
				};

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name.to_owned(), variable_type.to_owned(), intializer_value.to_owned()));
			}
			/* builtin macro calls */
			Identifier(identifier) if identifier.ends_with('!') => {
				/* check for ( */
				match iter.next() {
					Some(Operator(operator)) if *operator == '(' => (),
					_ => return Err((format!("expected operator '(' after macro '{identifier}'"), line))
				}

				let argument = match iter.next() {
					Some(StringLiteral(x)) => x,
					_ => return Err((format!("expected string literal in argument to macro '{identifier}'"), line))
				};
				
				/* check for ) */
				match iter.next() {
					Some(Operator(operator)) if *operator == ')' => (),
					_ => return Err(("expected operator ')' after macro argument".to_owned(), line))
				}

				ast.push(AstType::BuiltinMacroCall(identifier.to_owned(), argument.to_owned()));
			}
			_ => (),
		}
	}

	Ok(ast)
}