use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast::AstType;
use crate::ast::AstType::*;
use crate::lexer::{TokenType::*, token_to_string};

fn get_size_of_type(input: &str, line: i64) -> Result<(&'static str, u8), (String, i64)> {
	match (input) {
		"int8" => Ok(("byte", 1)),
		"int16" => Ok(("word", 2)),
		"int32" => Ok(("dword", 4)),
		"int64" => Ok(("qword", 8)),
		_ => return Err((format!("'{input}' is not a valid type"), line))
	}
} 

/* given a string, this function will insert that string into the datasection */
/* and return the identifier for it (like L0, L1, etc...) */
fn resolve_string_literal(datasect: &mut String, literal: &str) -> String {
	static mut LITERALS_AMOUNT: i64 = 0;

	/* mutating a static mut is unsafe because it can cause data races with multithreading */
	/* but because this program is singlethreaded (for now) this is perfectly safe */
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

	/* key is the variable name, value is its address */
	let mut local_variables: HashMap<String, String> = HashMap::new();
	let mut functions: HashSet<String> = HashSet::new(); 
	
	/* not even gonna bother explaining this */
	let mut stacksize = 0;
	let mut stackspace = 0;

	let mut calls_funcs = false;
	let mut stack_subtraction_index = 0;

	for i in iter {
		match i {
			AstType::Newline => line += 1,
			/* function stuffs */
			FunctionDefinition(name, _) => {
				textsect.push_str(&format!("global {name}\n{name}:\n"));
				textsect.push_str("\tpush rbp\n\tmov rbp, rsp\n\n");

				stack_subtraction_index = textsect.len() - 1;
				functions.insert(name.to_string());
			},
			FunctionCall(name, _) => {
				if (!functions.contains(name.to_owned())) {
					return Err((format!("undefined function '{name}'"), line))
				}
				
				textsect.push_str(&format!("\tcall {name}\n\n"));
				calls_funcs = true;
			},
			ScopeEnd => {
				if (calls_funcs && stackspace != 0) {
					textsect.push_str("\tleave\n");
				}
				else {
					textsect.push_str("\tpop rbp\n");
				}
				
				/* we want to subtract the value of stackspace from rsp if we call other functions */
				/* and if the aren't any local variables in the current function */
				if (calls_funcs && stackspace != 0) {
					textsect.insert_str(stack_subtraction_index, &format!("\tsub rsp, {stackspace}\n"));
				}

				textsect.push_str("\tret\n\n");
				stackspace = 0;
				stacksize = 0;
				calls_funcs = false;

				local_variables.clear();
			},
			/* variable stuffs */
			VariableDefinition(name, vartype, initval) => {
				let (word, bytesize) = get_size_of_type(vartype, line)?;

				stacksize += bytesize;
				/* grow stackspace if we ran out of space */
				if (stacksize > stackspace) {
					stackspace += 16;
				}

				let addr = format!("[rbp-{stacksize}]");

				textsect.push_str(&format!("\tmov {word} {addr}, {initval}\n"));
				local_variables.insert(name.to_string(), addr);
			},
			/* macro stuffs */
			MacroCall("asm!", args) => {
				let instruction = match args[0] {
					StringLiteral(ref x) => x,
					err => return Err((format!("expected token type to be a string literal, not {}", token_to_string(err)), line))
				};

				textsect.push_str(&format!("\t{}\n", instruction));
			},
			MacroCall("syscall!", args) => {
				for (i, v) in args.iter().enumerate() {
					let v = match (v) {
						IntLiteral(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut datasect, x),
						Identifier(varname) => { 
							match local_variables.get(varname) {
								Some(varaddr) => varaddr.to_owned(),
								None => return Err((format!("variable '{varname}' is not defined in the current scope"), line))
							}
						},

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
			},
			MacroCall(name, _) => {
				return Err((format!("macro '{}' does not exist", name), line));
			},
		}
	}

	Ok(datasect + &textsect)
}
