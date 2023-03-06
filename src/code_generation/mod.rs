use std::collections::HashMap;

mod registers;

use crate::parser::AstType::{self, *};
use crate::parser::expressions;
use crate::parser::expressions::Expression::*;

use registers::*;

/* this will have more fields in the future, like the return value */
struct Function<'a> {
	arg_types: &'a Vec<String>, /* types of paramaters, but not the names of the paramaters */
	return_type: &'a Option<String>
}

struct Variable {
	addr: String,
	vartype: String
}

/* the mutable state of the current function */
/* stuff like local variables, the size of the stack, etc */
#[derive(Default)]
struct FunctionState {
	local_variables: HashMap<String, Variable>,
	stacksize: i32,
	stack_subtraction_index: usize,

	calls_funcs: bool,
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
	function_state: &mut FunctionState,

	textsect: &mut String,

	name: &str, 
	vartype: &str, 
	initval: Option<&str>
) -> Result<(), (String, i64)> {
	let (word, bytesize) = get_size_of_type(vartype, line)?;
	function_state.stacksize += bytesize;

	let addr = format!("[rbp-{}]", function_state.stacksize);
	if let Some(initval) = initval {
		textsect.push_str(&format!("\tmov {word} {addr}, {initval}\n"));
	}

	function_state.local_variables.insert(name.to_string(), Variable {
		addr, 
		vartype: vartype.to_owned() 
	});

	Ok(())
}

fn call_function(
	name: &str, 
	args: &Vec<expressions::Expression>, 

	functions: &HashMap<String, Function>,
	current_function: &mut FunctionState,

	line: i64,

	textsect: &mut String,
	datasect: &mut String

) -> Result<(), (String, i64)> {
	let function = match functions.get(name) {
		Some(x) => x,
		None => return Err((format!("undefined function '{name}'"), line))
	};

	if (args.len() != function.arg_types.len()) {
		/* weird looking if statment is here so we dont produce an error message with broken english */
		return Err((format!("function '{name}' accepts {} arguments but {} {} given", function.arg_types.len(), args.len(), if (args.len() == 1) {
			"was"
		} 
		else {
			"were"
		}), line))
	}

	/* insert arguments to their respective registers */
	for (i, v) in args.iter().enumerate() {
		match (v) {
			Numerical(x) => {
				let (word, _) = get_size_of_type(&function.arg_types[i], line)?;
				let register = get_register_call(i, word, line)?;

				textsect.push_str(&format!("\tmov {register}, {x}\n"));
			},
			StringLiteral(x) => {
				let register = get_register_call(i, "qword", line)?;
				let identifier = resolve_string_literal(datasect, x);

				textsect.push_str(&format!("\tmov {register}, {identifier}\n"));
			},
			Variable(varname) => { 
				match current_function.local_variables.get(varname) {
					Some(var) => {
						if (function.arg_types[i] != var.vartype) {
							return Err((format!("function '{name}' accepts type {} as paramater {} but the type of '{varname}' is {}", function.arg_types[i], i + 1, var.vartype), line))
						}

						let (word, _) = get_size_of_type(&var.vartype, line)?;
						let register = get_register_call(i, word, line)?;

						if (word == "dword" || word == "qword") {
							textsect.push_str(&format!("\tmov {register}, {word} {}\n", var.addr));
						}
						else {
							let register32 = get_register_call(i, "dword", line)?;
							textsect.push_str(&format!("\tmovsx {register32}, {word} {}\n", var.addr));
						}
					},
					None => return Err((format!("variable '{varname}' is not defined in the current scope"), line))
				}
			},

			err => return Err((format!("expected either an int literal, string literal or identifier in call to function '{name}', but got {}", expressions::expression_to_string(err)), line))
		};
	}
	
	textsect.push_str(&format!("\tcall {name}\n"));
	current_function.calls_funcs = true;

	Ok(())
}

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn generate(input: &[AstType]) -> Result<String, (String, i64)> {
	let iter = input.iter();

	let mut line: i64 = 1;

	let mut datasect = String::from("section .data\n");
	let mut textsect = String::from("section .text\n\n");

	let mut functions: HashMap<String, Function> = HashMap::new();
	let mut current_function = FunctionState::default();

	for i in iter {
		match i {
			AstType::Newline => line += 1,
			/* ---------------------------- */
			/*     function definitions     */
			/* ---------------------------- */
			FunctionDefinition(name, args) => {
				textsect.push_str(&format!("global {name}\n{name}:\n"));
				textsect.push_str("\tpush rbp\n\tmov rbp, rsp\n\n");
				
				current_function.stack_subtraction_index = textsect.len() - 1;

				/* add arguments to the stack */
				for i in 0..args.0.len() {
					let (word, _) = get_size_of_type(&args.1[i], line)?;
					add_variable(
						line,
	
						&mut current_function,
						&mut textsect, 
	
						&args.0[i], &args.1[i], Some(get_register_definition(i, word, line)?)
					)?;
				}
				
				functions.insert(name.to_string(), Function { arg_types: &args.1, return_type: &None });
			},
			/* --------------------------- */
			/*     function prototypes     */
			/* --------------------------- */
			FunctionPrototype(name, args, return_type) => {
				textsect.push_str(&format!("extern {name}\n"));

				functions.insert(name.to_string(), Function { arg_types: args, return_type });
			}
			/* -------------------------- */
			/*      function calling      */
			/* -------------------------- */
			AstType::FunctionCall(name, args) => {
				call_function(name, args, 
					&functions, 
					&mut current_function, 
					line, 
					
					&mut textsect, 
					&mut datasect
				)?;
				textsect.push('\n');
			},
			/* -------------------------- */
			/*        function end        */
			/* -------------------------- */
			ScopeEnd => {
				if (current_function.calls_funcs && current_function.stacksize != 0) {
					textsect.push_str("\tleave\n");
				}
				else {
					textsect.push_str("\tpop rbp\n");
				}
				
				/* we want to subtract the value of stackspace from rsp if we call other functions */
				/* and if the aren't any local variables in the current function */
				if (current_function.calls_funcs && current_function.stacksize != 0) {
					textsect.insert_str(current_function.stack_subtraction_index, &format!("\tsub rsp, {}\n", current_function.stacksize));
				}

				textsect.push_str("\tret\n\n");
				current_function = FunctionState::default();
			},
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
			VariableDefinition(name, vartype, initval) => {
				let initializer = match (initval) {
					Numerical(x) => x.to_string(),
					StringLiteral(x) => resolve_string_literal(&mut datasect, x),
					Variable(varname) => {
						/* we move the variable to a temporary register and then pass that into add_variable */
						/* we have to use a temp register because we can't mov a memory location to another memory location obv */
						let var = match current_function.local_variables.get(varname) {
							Some(x) => x,
							None => return Err((format!("attempted to create variable with initializer value of variable '{varname}', but such variable does not exist"), line))
						};

						/* mismatch in types */
						if (var.vartype != *vartype) {
							return Err((format!("mismatch of types in variable decleration of '{name}', type of '{name}' is '{vartype}', yet the type that its being assigned to is the value of the variable '{varname}', which is of type '{}'", var.vartype), line));
						}

						let (word, _) = get_size_of_type(&var.vartype, line)?;
						let register = get_accumulator(word);

						textsect.push_str(&format!("\tmov {register}, {word} {}\n", var.addr));
						
						register.to_owned()
					},
					FunctionCallExpression(function_name, args) => {
						call_function(function_name, args, 
							&functions, 
							&mut current_function, 
							line, 
							
							&mut textsect, 
							&mut datasect
						)?;
						
						/* this unwrap will never fail because call_function will have handled it already at this point */
						let function = functions.get(function_name).unwrap();

						let return_type = match function.return_type {
							Some(x) => x,
							None => return Err((format!("attempted to get return value of function '{function_name}', but it does not return anything"), line))
						};

						if (return_type != vartype) {
							return Err((format!("attempted to assign return value of '{function_name}' to '{name}', but the return type of '{function_name}' is '{return_type}', while the type of the variable it was assigned to is '{vartype}'"), line));
						}

						let (word, _) = get_size_of_type(return_type, line)?;
						get_accumulator(word).to_owned()
					}
				};

				add_variable(
					line,

					&mut current_function,
					&mut textsect, 

					name, vartype, Some(&initializer)
				)?;

				textsect.push('\n');
			},
			/* -------------------------- */
			/*           macros           */
			/* -------------------------- */
			MacroCall("asm!", args) => {
				let instruction = match &args[0] {
					StringLiteral(ref x) => x,
					err => return Err((format!("expected token type to be a string literal, not {}", expressions::expression_to_string(err)), line))
				};

				textsect.push_str(&format!("\t{}\n", instruction));
			},
			MacroCall("syscall!", args) => {
				for (i, v) in args.iter().enumerate() {
					let v = match (v) {
						Numerical(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut datasect, x),
						Variable(varname) => { 
							match current_function.local_variables.get(varname) {
								Some(var) => {
									if (var.vartype != "i64") {
										return Err((format!("syscall! macro only accepts arguments of type i64, yet type of '{varname}' is {}", var.vartype), line));
									}
									var.addr.clone()
								}
								None => return Err((format!("variable '{varname}' is not defined in the current scope"), line))
							}
						},

						err => return Err((format!("expected either an int literal, string literal or identifier in call to macro syscall!, but got {}", expressions::expression_to_string(err)), line))
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
