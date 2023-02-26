use crate::ast::AstType;
use crate::ast::AstType::*;
use crate::lexer::TokenType::*;

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn parse(input: &[AstType]) -> String {
	let mut iter = input.iter();

	let datasect = String::from("section .data\n");
	let mut textsect = String::from("section .text\n");

	while let Some(i) = iter.next() {
		match i {
			FunctionDefinition(name, _) => {
				textsect.push_str(&format!("global {name}\n{name}:\n"));
			},
			BuiltinMacroCall(name, args) => {
				if (*name == "asm!") {
					let instruction = match args[0] {
						StringLiteral(ref x) => x,
						_ => todo!("expected token type to be StringLiteral")
					};

					textsect.push_str(&format!("\t{}\n", instruction));
				}
			}
			_ => ()
		}
	}

	datasect + &textsect
}