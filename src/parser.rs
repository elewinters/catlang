use crate::ast::AstType;
use crate::ast::AstType::*;

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
			BuiltinMacroCall(name, arg) => {
				if (*name == "asm!") {
					textsect.push_str(&format!("\t{}\n", arg));
				}
			}
			_ => ()
		}
	}

	datasect + &textsect
}