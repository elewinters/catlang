use crate::ast::AstType;
use crate::ast::AstType::*;
use crate::lexer::{TokenType::*, token_to_string};

/* given a string, this function will insert that string into the datasection */
/* and return the identifier for it (like L0, L1, etc...) */
fn resolve_string_literal(datasect: &mut String, literal: &str) -> String {
	static mut LITERALS_AMOUNT: i64 = 0;

	unsafe {
		datasect.push_str(&format!("\tL{LITERALS_AMOUNT}: db `{literal}`, 0\n"));
		LITERALS_AMOUNT += 1;

		format!("L{}", LITERALS_AMOUNT - 1)
	}
}

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn parse(input: &[AstType]) -> Result<String, (String, i64)> {
	let mut line: i64 = 1;
	let iter = input.iter();

	let mut datasect = String::from("section .data\n");
	let mut textsect = String::from("section .text\n\n");

	for i in iter {
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
			},
			BuiltinMacroCall("syscall!", args) => {
				for (i, v) in args.iter().enumerate() {
					let v = match (v) {
						IntLiteral(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut datasect, x),
						Identifier(x) => x.to_owned(),

						err => return Err((format!("expected either an int literal, string literal or identifier in call to macro syscall!, but got {}", token_to_string(err)), line))
					};

					match i {
						0 => textsect.push_str(&format!("\tmov rax, {v}\n")),
						1 => textsect.push_str(&format!("\tmov rdi, {v}\n")),
						2 => textsect.push_str(&format!("\tmov rsi, {v}\n")),
						3 => textsect.push_str(&format!("\tmov rdx, {v}\n")),
						4 => textsect.push_str(&format!("\tmov r10, {v}\n")),
						5 => textsect.push_str(&format!("\tmov r8, {v}\n")),
						6 => textsect.push_str(&format!("\tmov r9, {v}\n")),
						_ => return Err((String::from("syscall! does not take more than 7 arguments"), line))
					}
				}
				
				textsect.push_str("\tsyscall\n\n");
			}
			BuiltinMacroCall(name, _) => {
				return Err((format!("macro '{}' does not exist", name), line));
			} 
			_ => ()
		}
	}

	Ok(datasect + &textsect)
}