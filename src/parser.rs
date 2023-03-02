use std::collections::HashMap;

use crate::ast::AstType;
use crate::ast::AstType::*;
use crate::lexer::{TokenType::*, token_to_string};

/* this will have more fields in the future, like the return value */
struct Function<'a> {
	args: &'a(Vec<String>, Vec<String>)
}

struct Variable {
	addr: String,
	vartype: String
}

fn get_size_of_type(input: &str, line: i64) -> Result<(&'static str, i32), (String, i64)> {
	match (input) {
		"int8" => Ok(("byte", 1)),
		"int16" => Ok(("word", 2)),
		"int32" => Ok(("dword", 4)),
		"int64" => Ok(("qword", 8)),
		_ => return Err((format!("'{input}' is not a valid type"), line))
	}
} 

/* returns a register in system V amd64 calling convention, the default for most x86_64 compilers */
fn get_register_from_argc(argument_count: usize, line: i64) -> Result<&'static str, (String, i64)> {
	match (argument_count) {
		0 => Ok("rdi"),
		1 => Ok("rsi"),
		2 => Ok("rdx"),
		3 => Ok("rcx"),
		4 => Ok("r8"),
		5 => Ok("r9"),
		_ => Err((String::from("too many arguments to function, functions can only up to 6 arguments at the moment"), line))
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

/* adds a variable to the local_variables hashmap */
fn add_variable(
	line: i64,
	stacksize: &mut i32,

	textsect: &mut String,
	local_variables: &mut HashMap<String, Variable>,

	name: &str, 
	vartype: &str, 
	initval: Option<&str>) -> Result<(), (String, i64)> 
{
	let (word, bytesize) = get_size_of_type(vartype, line)?;

	*stacksize += bytesize;

	let addr = format!("[rbp-{stacksize}]");

	if let Some(initval) = initval {
		textsect.push_str(&format!("\tmov {word} {addr}, {initval}\n"));
	}

	local_variables.insert(name.to_string(), Variable {
		addr, 
		vartype: vartype.to_owned() 
	});

	Ok(())
}

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn parse(input: &[AstType]) -> Result<String, (String, i64)> {
	let mut line: i64 = 1;
	let iter = input.iter();

	let mut datasect = String::from("section .data\n");
	let mut textsect = String::from("section .text\n\n");

	/* key is the variable name, value is its address */
	let mut local_variables: HashMap<String, Variable> = HashMap::new();
	let mut functions: HashMap<String, Function> = HashMap::new(); 
	
	/* not even gonna bother explaining this */
	let mut stacksize = 0;

	let mut calls_funcs = false;
	let mut stack_subtraction_index = 0;

	for i in iter {
		match i {
			AstType::Newline => line += 1,
			/* function stuffs */
			FunctionDefinition(name, args) => {
				textsect.push_str(&format!("global {name}\n{name}:\n"));
				textsect.push_str("\tpush rbp\n\tmov rbp, rsp\n\n");
				
				stack_subtraction_index = textsect.len() - 1;

				/* add arguments to the stack */
				for i in 0..args.0.len() {
					add_variable(
						line,
	
						&mut stacksize,
						&mut textsect, 
						&mut local_variables,
	
						&args.0[i], &args.1[i], Some(get_register_from_argc(i, line)?)
					)?;
				}
				
				functions.insert(name.to_string(), Function { args });
			},
			FunctionCall(name, args) => {
				let function = match functions.get(*name) {
					Some(x) => x,
					None => return Err((format!("undefined function '{name}'"), line))
				};

				if (args.len() != function.args.0.len()) {
					/* weird looking if statment is here so we dont produce an error message with broken english */
					return Err((format!("function '{name}' accepts {} arguments but {} {} given", function.args.0.len(), args.len(), if (args.len() == 1) {
						"was"
					} 
					else {
						"were"
					}), line))
				}

				/* insert arguments to their respective registers */
				for (i, v) in args.iter().enumerate() {
					let v = match (v) {
						IntLiteral(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut datasect, x),
						Identifier(varname) => { 
							match local_variables.get(varname) {
								Some(var) => {
									if (function.args.1[i] != var.vartype) {
										return Err((format!("function '{name}' accepts type {} as paramater {} but the type of '{varname}' is {}", function.args.1[i], i + 1, var.vartype), line))
									}
									var.addr.clone()
								},
								None => return Err((format!("variable '{varname}' is not defined in the current scope"), line))
							}
						},

						err => return Err((format!("expected either an int literal, string literal or identifier in call to function '{name}', but got {}", token_to_string(err)), line))
					};

					textsect.push_str(&format!("\tmov {}, {v}\n", get_register_from_argc(i, line)?));
				}
				
				textsect.push_str(&format!("\tcall {name}\n\n"));
				calls_funcs = true;
			},
			ScopeEnd => {
				if (calls_funcs && stacksize != 0) {
					textsect.push_str("\tleave\n");
				}
				else {
					textsect.push_str("\tpop rbp\n");
				}
				
				/* we want to subtract the value of stackspace from rsp if we call other functions */
				/* and if the aren't any local variables in the current function */
				if (calls_funcs && stacksize != 0) {
					textsect.insert_str(stack_subtraction_index, &format!("\tsub rsp, {stacksize}\n"));
				}

				textsect.push_str("\tret\n\n");
				stacksize = 0;
				calls_funcs = false;

				local_variables.clear();
			},
			/* variable stuffs */
			VariableDefinition(name, vartype, initval) => {
				add_variable(
					line,

					&mut stacksize,
					&mut textsect, 
					&mut local_variables,

					name, vartype, Some(initval)
				)?;
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
								Some(var) => {
									if (var.vartype != "int64") {
										return Err((format!("syscall! macro only accepts arguments of type int64, yet type of '{varname}' is {}", var.vartype), line));
									}
									var.addr.clone()
								}
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
