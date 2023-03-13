use std::collections::HashMap;

mod registers;

use crate::parser::AstType::{self, *};
use crate::lexer::TokenType::{self, *};

use crate::parser::{process_function_parameters, Expression};

macro_rules! warn {
	($fmt:expr, $line:expr) => {
		{
			println!("[line {}] {}", $line, String::from("catlang: \x1b[33mwarning:\x1b[0m ") + &$fmt);
		}
	}
}

use registers::*;
#[derive(Clone)]
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
	current_function: FunctionState,

	labels: i64
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

/* evaluates an expression (aka a list of tokens) and returns the result where its stored at (or just its literal value if no operations were done on it) */
/* an example input of [5, +, 5, +, strlen, (, "12345", )] with expected_type as i32 would give you the result 'r11d', which is where the result of this expression is stored at (the result is 15 by the way) */
/* another example input of ["hello world"] would return the identifier for this string literal, so something like L0 or L1 */
/* another example input of [5] would return 5 */
fn eval_expression(state: &mut State, expr: &Expression, expected_type: &str) -> Result<String, (String, i64)> {
	let mut iter = expr.iter();

	/* what this function does is it evaluates a single element of an expression, a sort of "miniexpression" */
	/* so if you feed it '5' it will return '5' */
	/* if you feed it the string literal "hello" it will return L0, L1, etc */
	/* if you feed it 'strlen, (, "hi", )' it will return rax/eax */
	/* if you feed it 'var' it will return its address on the stack (like [rbp-16]) */
	fn eval_miniexpression(state: &mut State, iter: &mut core::slice::Iter<TokenType>, expected_type: &str) -> Result<String, (String, i64)> {
		Ok(match (iter.next(), iter.clone().peekable().peek()) {
			(Some(IntLiteral(x)), _) => x.to_string(),
			(Some(StringLiteral(x)), _) => resolve_string_literal(&mut state.datasect, x),

			/* function calls */
			(Some(Identifier(function_name)), Some(Operator('('))) => {
				iter.next(); /* strip ( */

				let args = process_function_parameters(iter);
				call_function(state, function_name, &args)?;
		
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
				get_accumulator(&word).to_owned()
			}

			/* variables */
			(Some(Identifier(x)), _) => {
				/* we move the variable to a temporary register and then pass that into add_variable */
				/* we have to use a temp register because we can't mov a memory location to another memory location obv */
				let var = match state.current_function.local_variables.get(x) {
					Some(x) => x,
					None => return Err((format!("variable '{x}' is not defined in the current scope"), state.line))
				};
	
				/* mismatch in types */
				if (var.vartype != expected_type) {
					return Err((format!("expected expression to evaluate to type '{expected_type}', but the type of '{x}' is '{}'", var.vartype), state.line));
				}
	
				var.addr.clone()
			}
			
			(Some(x), _) => return Err((format!("expected int literal, string literal or identifier in expression, but got {x}"), state.line)),
			(None, _) => return Err((String::from("expected int literal, string literal, or identifier in expression, but got nothing"), state.line))
		})
	}

	let (word, _) = get_size_of_type(expected_type, state.line)?;
	let root_register = get_accumulator2(&word);

	let root_value = eval_miniexpression(state, &mut iter, expected_type)?;
	
	/* if the expression only has one element we just return its root */
	if (iter.clone().peekable().peek().is_none()) {
		return Ok(root_value);
	}

	/* if it has multiple elements we move it to the accumulator (unless its already there) */
	if (root_value != root_register) {
		state.textsect.push_str(&format!("\tmov {root_register}, {root_value}\n"));
	}
	
	/* now we do all sorts of operations on the accumulator */
	while let Some(i) = iter.next() {
		let val = eval_miniexpression(state, &mut iter, expected_type)?;
		match i {
			Operator('+') => {
				state.textsect.push_str(&format!("\tadd {root_register}, {val}\n"));
			},
			Operator('-') => {
				state.textsect.push_str(&format!("\tsub {root_register}, {val}\n"));
			}
			Operator('*') => {
				state.textsect.push_str(&format!("\timul {root_register}, {val}\n"));
			}
			Operator('/') => {
				warn!("divison is an unstable feature", state.line);

				let accumulator = get_accumulator(&word);
				let accumulator3 = get_accumulator3(&word);

				/* this is where we have to place the first value of division operation (accumulator) */
				state.textsect.push_str("\txor rax, rax\n");
				/* and this is where our divisor goes (accumulator3) */
				state.textsect.push_str("\txor rbx, rbx\n");
				/* clear out rdx before division (if we dont do this we will Crash the Fucking Program) */
				state.textsect.push_str("\tcdq\n");
				

				state.textsect.push_str(&format!("\tmov {accumulator}, {word} {root_register}\n"));
				state.textsect.push_str(&format!("\tmov {accumulator3}, {val}\n"));
				state.textsect.push_str(&format!("\tidiv {accumulator3}\n"));

				state.textsect.push_str(&format!("\tmov {root_register}, {accumulator}\n"));
			}

			err => return Err((format!("unexpected {err} in expression evaluation"), state.line))
		}
	}

	/* once we're done we return it */
	Ok(root_register.to_owned())
}

/* adds a variable to the local_variables hashmap */
fn add_variable(state: &mut State, name: &str, vartype: &str, initval: Option<&str>) -> Result<(), (String, i64)> {
	/* we dont make the 2nd value of this tuple '_' for once! */
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
	let function = match state.functions.get(name).cloned() {
		Some(x) => x,
		None => return Err((format!("undefined function '{name}'"), state.line))
	};

	/* check if the caller provided enough arguments */
	if (args.len() != function.arg_types.len()) {
		/* weird looking if statment is here so we dont produce an error message with broken english */
		return Err((format!("function '{name}' accepts {} arguments but {} {} given", function.arg_types.len(), args.len(), if (args.len() == 1) {
			"was"
		} 
		else {
			"were"
		}), state.line))
	}

	/* we set up a queue for passing arguments to our function, after we're done evaluating all of our expressions we */
	/* insert the 2nd value of the tuple (the expression evaluation) to the first value of the tuple (the register) */

	/* the reason why we have this queue and why we dont just push the expr onto the register in the loop below, is that an expression evaluation can also call other functions */
	/* like in an expression like this [sum(100, sum(50, 50))] */
	/* without this queue passing all of the arguments to their registers would get totally messed up */
	let mut args_queue: Vec<(&'static str, String)> = Vec::new();

	/* insert arguments to the queue */
	for (i, v) in args.iter().enumerate() {
		let expr_evaluation = eval_expression(state, v, &function.arg_types[i])?;

		let (word, _) = get_size_of_type(&function.arg_types[i], state.line)?;
		let register = get_register(i, &word, state.line)?;

		args_queue.push((register, expr_evaluation));
	}

	/* now pass the arguments into their respective registers */
	for (register, expr_eval) in args_queue {
		state.textsect.push_str(&format!("\tmov {register}, {expr_eval}\n"));
	}

	/* other functions may modify r11, which cannot happen as that will mess up expressions*/
	state.textsect.push_str(&format!("\n\tpush r11\n"));
	state.textsect.push_str(&format!("\tcall {name}\n"));
	state.textsect.push_str(&format!("\tpop r11\n\n"));
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
		current_function: FunctionState::default(),

		labels: 0
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
					let register = get_register(i, &word, state.line)?;

					add_variable(&mut state, &args.0[i], &args.1[i], Some(register))?;
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
					state.textsect.push_str(&format!("\tmov {accumulator}, {return_value}\n"));
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
			/* ----------------------- */
			/*      if statements      */
			/* ----------------------- */
			IfStatement(expr1, operator, expr2) => {
				println!("{:?} [{:?}] {:?}", expr1, operator, expr2);
			},
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
			VariableDefinition(name, vartype, initexpr) => {
				/* now we evaluate expression */
				let value = eval_expression(&mut state, initexpr, vartype)?;
				add_variable(&mut state, name, vartype, Some(&value))?;

				state.textsect.push('\n');
			},
			/* -------------------------- */
			/*           macros           */
			/* -------------------------- */
			MacroCall("asm!", args) => {
				let instruction = match &args[0][0] {
					StringLiteral(ref x) => x,
					err => return Err((format!("expected token type to be a string literal, not {err}"), state.line))
				};

				state.textsect.push_str(&format!("\t{}\n", instruction));
			},
			MacroCall("syscall!", args) => {
				for (i, v) in args.iter().enumerate() {
					let v = match &(v[0]) {
						IntLiteral(x) => x.to_owned(),
						StringLiteral(x) => resolve_string_literal(&mut state.datasect, x),
						Identifier(varname) => { 
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
