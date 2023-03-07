use std::collections::HashMap;

mod registers;

use crate::parser::AstType::{self, *};
use crate::parser::expressions::Expression::{self, *};

use registers::*;
use registers::WordType::*;

struct Function<'a> {
	arg_types: &'a Vec<String>, /* types of paramaters, but not the names of the paramaters */
	return_type: &'a Option<String>
}

struct Variable {
	addr: String,
	vartype: String
}

/* this contains all of the state of the current function we're working with */
/* stuff like local variables, the size of the stack, etc */
/* this will always be mutable, and there will always only be one instance of it */
#[derive(Default)]
struct FunctionState {
	local_variables: HashMap<String, Variable>,
	return_type: Option<String>,
	stacksize: i32,
	stack_subtraction_index: usize,

	calls_funcs: bool,
}

/* contains all of the state that this module needs to preserve */
/* theres only ever one instance of this and its always mutable */
/* the reason why these arent just regular mut variables in the generate() function is because passing all of them to functions is a fucking pain */
/* so i turned them into a struct!! this is the cleanest way to do this i promise */
struct State<'a> {
	line: i64,

	datasect: String,
	textsect: String,

	functions: HashMap<String, Function<'a>>,
	current_function: FunctionState
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

/* evaluates an expression and returns the address/register of the result */
/* expected_type is the type that we expect this expression to eval to (i32, i64, etc) */
fn eval_expression(state: &mut State, expr: &Expression, expected_type: &str) -> Result<String, (String, i64)> {
	match (expr) {
		Numerical(x) => Ok(x.to_string()),
		StringLiteral(x) => Ok(resolve_string_literal(&mut state.datasect, x)),
		Variable(varname) => {
			/* we move the variable to a temporary register and then pass that into add_variable */
			/* we have to use a temp register because we can't mov a memory location to another memory location obv */
			let var = match state.current_function.local_variables.get(varname) {
				Some(x) => x,
				None => return Err((format!("attempted to create variable with initializer value of variable '{varname}', but such variable does not exist"), state.line))
			};

			/* mismatch in types */
			if (var.vartype != expected_type) {
				return Err((format!("expected expression to evaluate to type '{expected_type}', but the type of '{varname}' is '{}'", var.vartype), state.line));
			}

			let (word, _) = get_size_of_type(&var.vartype, state.line)?;
			let register = get_accumulator(&word);

			state.textsect.push_str(&format!("\tmov {register}, {word} {}\n", var.addr));
			
			Ok(register.to_owned())
		},
		FunctionCallExpression(function_name, args) => {
			call_function(state, function_name, args)?;
			
			/* this unwrap will never fail because call_function will have handled it already at this point */
			let function = state.functions.get(function_name).unwrap();

			let return_type = match function.return_type {
				Some(x) => x,
				None => return Err((format!("attempted to get return value of function '{function_name}', but it does not return anything"), state.line))
			};

			if (return_type != expected_type) {
				return Err((format!("expected expression to evaluate to '{expected_type}', but the return type of '{function_name}' is '{return_type}'"), state.line));
			}

			let (word, _) = get_size_of_type(return_type, state.line)?;
			Ok(get_accumulator(&word).to_owned())
		}
	}
}

/* adds a variable to the local_variables hashmap */
fn add_variable(state: &mut State, name: &str, vartype: &str, initval: Option<&str>) -> Result<(), (String, i64)> {
	/* we dont make the 2nd value '_' for once! */
	let (word, bytesize) = get_size_of_type(vartype, state.line)?;
	state.current_function.stacksize += bytesize;

	let addr = format!("[rbp-{}]", state.current_function.stacksize);
	if let Some(initval) = initval {
		state.textsect.push_str(&format!("\tmov {word} {addr}, {initval}\n"));
	}

	state.current_function.local_variables.insert(name.to_string(), Variable {
		addr, 
		vartype: vartype.to_owned() 
	});

	Ok(())
}

fn call_function(state: &mut State, name: &str, args: &Vec<Expression>) -> Result<(), (String, i64)> {
	let function = match state.functions.get(name) {
		Some(x) => x,
		None => return Err((format!("undefined function '{name}'"), state.line))
	};

	if (args.len() != function.arg_types.len()) {
		/* weird looking if statment is here so we dont produce an error message with broken english */
		return Err((format!("function '{name}' accepts {} arguments but {} {} given", function.arg_types.len(), args.len(), if (args.len() == 1) {
			"was"
		} 
		else {
			"were"
		}), state.line))
	}

	/* insert arguments to their respective registers */
	for (i, v) in args.iter().enumerate() {
		match (v) {
			Numerical(x) => {
				let (word, _) = get_size_of_type(&function.arg_types[i], state.line)?;
				let register = get_register(i, &word, state.line)?;

				state.textsect.push_str(&format!("\tmov {register}, {x}\n"));
			},
			StringLiteral(x) => {
				let register = get_register(i, &QuadWord, state.line)?;
				let identifier = resolve_string_literal(&mut state.datasect, x);

				state.textsect.push_str(&format!("\tmov {register}, {identifier}\n"));
			},
			Variable(varname) => { 
				match state.current_function.local_variables.get(varname) {
					Some(var) => {
						if (function.arg_types[i] != var.vartype) {
							return Err((format!("function '{name}' accepts type {} as paramater {} but the type of '{varname}' is {}", function.arg_types[i], i + 1, var.vartype), state.line))
						}

						let (word, _) = get_size_of_type(&var.vartype, state.line)?;
						let register = get_register(i, &word, state.line)?;

						match (word) {
							DoubleWord | QuadWord => {
								state.textsect.push_str(&format!("\tmov {register}, {word} {}\n", var.addr))
							},
							_ => {
								let register32 = get_register(i, &DoubleWord, state.line)?;
								state.textsect.push_str(&format!("\tmovsx {register32}, {word} {}\n", var.addr));
							}
						}
					},
					None => return Err((format!("variable '{varname}' is not defined in the current scope"), state.line))
				}
			},

			err => return Err((format!("expected either an int literal, string literal or identifier in call to function '{name}', but got {err}"), state.line))
		};
	}
	
	state.textsect.push_str(&format!("\tcall {name}\n"));
	state.current_function.calls_funcs = true;

	Ok(())
}

/* returns the assembly output on success, returns a string containing error information on failure */
pub fn generate(input: &[AstType]) -> Result<String, (String, i64)> {
	let iter = input.iter();

	let mut state = State {
		line: 1,
		datasect: String::from("section .data\n"),
		textsect: String::from("section .text\n\n"),

		functions: HashMap::new(),
		current_function: FunctionState::default()
	};

	for i in iter {
		match i {
			AstType::Newline => state.line += 1,
			/* ---------------------------- */
			/*     function definitions     */
			/* ---------------------------- */
			FunctionDefinition(name, args, return_type) => {
				state.textsect.push_str(&format!("global {name}\n{name}:\n"));
				state.textsect.push_str("\tpush rbp\n\tmov rbp, rsp\n\n");
				
				state.current_function.stack_subtraction_index = state.textsect.len() - 1;

				/* add arguments to the stack */
				for i in 0..args.0.len() {
					let (word, _) = get_size_of_type(&args.1[i], state.line)?;
					let register32 = get_register_32_or_64(i, &word, state.line)?;

					add_variable(&mut state, &args.0[i], &args.1[i], Some(register32))?;
				}
				
				state.functions.insert(name.to_string(), Function { arg_types: &args.1, return_type });
				state.current_function.return_type = return_type.clone();
			},
			/* --------------------------- */
			/*     function prototypes     */
			/* --------------------------- */
			FunctionPrototype(name, args, return_type) => {
				state.textsect.push_str(&format!("extern {name}\n"));

				state.functions.insert(name.to_string(), Function { arg_types: args, return_type });
			}
			/* -------------------------- */
			/*      function calling      */
			/* -------------------------- */
			FunctionCall(name, args) => {
				call_function(&mut state, name, args)?;
				state.textsect.push('\n');
			},
			/* ------------------------ */
			/*    function returning    */
			/* ------------------------ */
			ReturnStatement(expr) => {
				let return_type = match state.current_function.return_type {
					Some(ref x) => x.clone(),
					None => return Err((String::from("attempted to return from function that does not return anything, did you forget to specify the return type in the signature?"), state.line))
				};

				let return_value = eval_expression(&mut state, expr, &return_type)?;

				let (word, _) = get_size_of_type(&return_type, state.line)?;
				let accumulator = get_accumulator(&word);

				/* the return_value can sometimes be the accumulator */
				/* which means that we'll be moving rax to rax, which is just unnecessary */
				if (accumulator != return_value) {
					state.textsect.push_str(&format!("\tmov {accumulator}, {word} {return_value}\n"));
				}
			}
			/* -------------------------- */
			/*        function end        */
			/* -------------------------- */
			ScopeEnd => {
				if (state.current_function.calls_funcs && state.current_function.stacksize != 0) {
					state.textsect.push_str("\tleave\n");
				}
				else {
					state.textsect.push_str("\tpop rbp\n");
				}
				
				/* we want to subtract the value of stackspace from rsp if we call other functions */
				/* and if the aren't any local variables in the current function */
				if (state.current_function.calls_funcs && state.current_function.stacksize != 0) {
					state.textsect.insert_str(state.current_function.stack_subtraction_index, &format!("\tsub rsp, {}\n", state.current_function.stacksize));
				}

				state.textsect.push_str("\tret\n\n");
				state.current_function = FunctionState::default();
			},
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
			VariableDefinition(name, vartype, initval) => {
				let initializer = eval_expression(&mut state, initval, vartype)?;

				add_variable(&mut state, name, vartype, Some(&initializer))?;

				state.textsect.push('\n');
			},
			/* -------------------------- */
			/*           macros           */
			/* -------------------------- */
			MacroCall("asm!", args) => {
				let instruction = match &args[0] {
					StringLiteral(ref x) => x,
					err => return Err((format!("expected token type to be a string literal, not {err}"), state.line))
				};

				state.textsect.push_str(&format!("\t{}\n", instruction));
			},
			MacroCall("syscall!", args) => {
				for (i, v) in args.iter().enumerate() {
					let v = match (v) {
						Numerical(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut state.datasect, x),
						Variable(varname) => { 
							match state.current_function.local_variables.get(varname) {
								Some(var) => {
									if (var.vartype != "i64") {
										return Err((format!("syscall! macro only accepts arguments of type i64, yet type of '{varname}' is {}", var.vartype), state.line));
									}
									var.addr.clone()
								}
								None => return Err((format!("variable '{varname}' is not defined in the current scope"), state.line))
							}
						},

						err => return Err((format!("expected either an int literal, string literal or identifier in call to macro syscall!, but got {err}"), state.line))
					};

					match i {
						0 => state.textsect.push_str(&format!("\tmov rax, {v}\n")),
						1 => state.textsect.push_str(&format!("\tmov rdi, {v}\n")),
						2 => state.textsect.push_str(&format!("\tmov rsi, {v}\n")),
						3 => state.textsect.push_str(&format!("\tmov rdx, {v}\n")),
						4 => state.textsect.push_str(&format!("\tmov r10, {v}\n")),
						5 => state.textsect.push_str(&format!("\tmov r8, {v}\n")),
						6 => state.textsect.push_str(&format!("\tmov r9, {v}\n")),
						_ => return Err((String::from("syscall! does not take more than 7 arguments"), state.line))
					}
				}
				
				state.textsect.push_str("\tsyscall\n\n");
			},
			MacroCall(name, _) => {
				return Err((format!("macro '{}' does not exist", name), state.line));
			},
		}
	}

	Ok(state.datasect + &state.textsect)
}
