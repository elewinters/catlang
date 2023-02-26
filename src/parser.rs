use crate::ast::AstType;
use crate::ast::AstType::*;
use crate::lexer::{TokenType::*, token_to_string};

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn parse(input: &[AstType]) -> Result<String, (String, i64)> {
	let mut line: i64 = 1;
	let mut iter = input.iter();

	let datasect = String::from("section .data\n");
	let mut textsect = String::from("section .text\n");

	while let Some(i) = iter.next() {
		match i {
			AstType::Newline => line += 1,
			FunctionDefinition(name, _) => {
				textsect.push_str(&format!("global {name}\n{name}:\n"));
			},
			BuiltinMacroCall("asm!", args) => {
				let instruction = match args[0] {
					StringLiteral(ref x) => x,
					err => return Err((format!("expected token type to be a string literal, not {}", token_to_string(err)), line))
				};

				textsect.push_str(&format!("\t{}\n", instruction));
			}
			_ => ()
		}
	}

	Ok(datasect + &textsect)
}