use crate::lexer::TokenType;
use crate::lexer::TokenType::*;

use std::collections::HashMap;

pub enum AstType {
	/* function name, optional hashmap of paramaters (key being the identifier, value being the datatype) */
	FunctionDefinition(String, Option<HashMap<String, String>>)
}

pub fn ast(input: &Vec<TokenType>) -> Result<Vec<AstType>, String> {
	let mut ast: Vec<AstType> = Vec::new();

	let mut iter = input.iter();
	while let Some(i) = iter.next() {
		match i {
			/* function definitions */
			Keyword(keyword) if keyword == "fn" => {
				let next = match iter.next() {
					Some(x) => x,
					None => return Err("expected a token after the function keyword".to_owned())
				};

				let function_name = match next {
					Identifier(x) => x,
					err => return Err(format!("expected identifier after function keyword, not {}", err.human_readable()))
				};

				ast.push(AstType::FunctionDefinition(function_name.to_owned(), None));
			},
			/* variable declerations */
			Keyword(keyword) if keyword == "let" => {

			}
			_ => (),
		}
	}

	Ok(ast)
}