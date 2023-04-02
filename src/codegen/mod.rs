use std::collections::HashMap;

mod registers;
use registers::*;

mod macros;
mod expressions;

use expressions::*;

use crate::parser::AstType::{self, *};
use crate::lexer::Token::{self, *};
use crate::lexer::Operator::*;

use crate::parser::{process_function_parameters, Expression, ComparisonOperator};

/* ------------------------------ */
/*           structures           */
/* ------------------------------ */

#[derive(Clone, PartialEq)]
pub struct DataType {
	string: String, /* i8, i16, i32, i64 */
	word: WordType,
	byte_size: i32
}

#[derive(Clone)]
struct Variable {
	addr: String,
	vartype: DataType
}

#[derive(Clone)]
struct Function {
	arg_types: Vec<String>, /* types of paramaters, but not the names of the paramaters */
	return_type: Option<DataType>
}

/* this contains all of the state of the current function we're working with */
/* stuff like local variables, the size of the stack, etc */
/* this will always be mutable, and there will always only be one instance of it */
#[derive(Clone)]
struct CurrentFunctionState {
	name: String,

	local_variables: HashMap<String, Variable>,
	return_type: Option<DataType>,
	stacksize: i32,
	stackspace: i32,

	calls_funcs: bool,
	returns: bool,
}

/* contains all of the state that this module needs to preserve */
/* theres only ever one instance of this and its always mutable */
/* the reason why these arent just regular mut variables in the generate() function is because passing all of them to functions is a fucking pain */
/* so i turned them into a struct!! this is the cleanest way to do this i promise */
#[derive(Default, Clone)]
pub struct State {
	line: i64,

	pub datasect: String,
	pub textsect: String,

	functions: HashMap<String, Function>,
	current_function: CurrentFunctionState,

	labels: i64,
}

/* ------------------------------- */
/*      trait implementations      */
/* ------------------------------- */

impl DataType {
	fn new(input: &str, line: i64) -> Result<Self, (String, i64)> {
		Ok(match input {
			"i8" => Self { string: input.to_owned(), word: WordType::Byte, byte_size: 1 },
			"i16" => Self { string: input.to_owned(), word: WordType::Word, byte_size: 2 },
			"i32" => Self { string: input.to_owned(), word: WordType::DoubleWord, byte_size: 4 },
			"i64" => Self { string: input.to_owned(), word: WordType::QuadWord, byte_size: 8 },
			
			_ => return Err((format!("'{input}' is not a valid type"), line)) 
		})
	}
}

impl Default for CurrentFunctionState {
	fn default() -> Self {
		Self {
			name: String::new(),
			local_variables: HashMap::new(),
			return_type: None,
			
			/* stacksize needs to start at 8 because whenever we push rbx, [rbp-8] becomes the location of rbx */
			/* this is bad because whenever we make a variable we will start at [rbp-4] or [rbp-8] */
			/* meaning we will overwrite rbx on the stack */
			/* took me a bit to figure this out */
			stacksize: 8,
			stackspace: 0,

			calls_funcs: false,
			returns: false,
		}
	}
}

/* -------------------------------- */
/*           module logic           */
/* -------------------------------- */

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
fn add_variable(state: &mut State, name: &str, vartype: &DataType, initval: Option<&str>) -> Result<(), (String, i64)> {
	state.current_function.stacksize += vartype.byte_size;
	
	if (state.current_function.stacksize > state.current_function.stackspace + 8) {
		state.current_function.stackspace += 16
	}

	let addr = format!("[rbp-{}]", state.current_function.stacksize);
	if let Some(initval) = initval {
		state.textsect.push_str(&format!("\tmov {} {addr}, {initval}\n", vartype.word));
	}

	state.current_function.local_variables.insert(name.to_string(), Variable {
		addr, 
		vartype: vartype.clone()
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
		let expr_evaluation = eval_expression(state, v, &DataType::new(&function.arg_types[i], state.line)?)?;

		let argtype = DataType::new(&function.arg_types[i], state.line)?;
		let register = get_register(i, &argtype.word, state.line)?;

		args_queue.push((register, expr_evaluation));
	}

	/* now pass the arguments into their respective registers */
	for (register, expr_eval) in args_queue {
		state.textsect.push_str(&format!("\tmov {register}, {expr_eval}\n"));
	}

	state.textsect.push_str(&format!("\tcall {name}\n\n"));
	state.current_function.calls_funcs = true;

	Ok(())
}

/* returns the state of the program on success, returns a string containing error information on failure */
pub fn generate(state: &mut State, input: &[AstType]) -> Result<(), (String, i64)> {
	let iter = input.iter();

	for i in iter {
		match i {
			AstType::Newline => state.line += 1,
			/* ---------------------------- */
			/*     function definitions     */
			/* ---------------------------- */
			FunctionDefinition(name, args, return_type, body) => {
				state.textsect.push_str(&format!("global {name}\n{name}:\n"));
				state.textsect.push_str("\tpush rbp\n");
				state.textsect.push_str("\tmov rbp, rsp\n\n");

				let stack_subtraction_index = state.textsect.len() - 1;

				/* add arguments to the stack */
				for i in 0..args.0.len() {
					let datatype = DataType::new(&args.1[i], state.line)?;
					let register = get_register(i, &datatype.word, state.line)?;

					add_variable(state, &args.0[i], &datatype, Some(register))?;
				}

				let return_type = match return_type {
					Some(x) => Some(DataType::new(x, state.line)?),
					None => None,
				};
				
				state.functions.insert(name.to_string(), Function { arg_types: args.1.to_vec(), return_type: return_type.clone() });
				state.current_function.return_type = return_type;
				state.current_function.name = name.clone();

				/* parse and append the body of the funnction */
				generate(state, body)?;

				if (state.current_function.returns) {
					state.textsect.push_str(&format!("\n.ret_{}:", state.current_function.name));
				}

				/* we want to subtract the value of stackspace + 8 (+8 because of rbx) from rsp if we call other functions */
				/* and if the aren't any local variables/arguments in the current function */
				if (state.current_function.calls_funcs && state.current_function.stacksize != 0) {
					state.textsect.push_str("\n\tpop rbx\n");

					state.textsect.insert_str(stack_subtraction_index, &format!("\tsub rsp, {}\n", state.current_function.stackspace + 8));
					state.textsect.insert_str(stack_subtraction_index, "\tpush rbx\n");

					state.textsect.push_str("\tleave\n");
				}
				else {
					state.textsect.push_str("\n\tpop rbx\n");
					state.textsect.insert_str(stack_subtraction_index, "\tpush rbx\n");
					
					state.textsect.push_str("\tpop rbp\n");
				}
				
				state.textsect.push_str("\tret\n\n");
				state.current_function = CurrentFunctionState::default();
			},
			/* --------------------------- */
			/*     function prototypes     */
			/* --------------------------- */
			FunctionPrototype(name, args, return_type) => {
				state.textsect.push_str(&format!("extern {name}\n"));

				let return_type = match return_type {
					Some(x) => Some(DataType::new(x, state.line)?),
					None => None,
				};

				state.functions.insert(name.to_string(), Function { arg_types: args.to_vec(), return_type });
			}
			/* -------------------------- */
			/*      function calling      */
			/* -------------------------- */
			FunctionCall(name, args) => {
				call_function(state, name, args)?;
			},
			/* ------------------------ */
			/*    function returning    */
			/* ------------------------ */
			ReturnStatement(expr) => {
				let return_type = match state.current_function.return_type {
					Some(ref x) => x.clone(),
					None => return Err((String::from("attempted to return from function that does not return anything, did you forget to specify the return type in the signature?"), state.line))
				};

				let return_value = eval_expression(state, expr, &return_type)?;
				let accumulator = get_accumulator(&return_type.word);

				/* the return_value can sometimes be the accumulator */
				/* which means that we'll be moving rax to rax, which is just unnecessary */
				if (accumulator != return_value) {
					state.textsect.push_str(&format!("\tmov {accumulator}, {return_value}\n"));
				}

				state.textsect.push_str(&format!("\tjmp .ret_{}\n", state.current_function.name));
				state.current_function.returns = true;
			}
			/* ----------------------- */
			/*      if statements      */
			/* ----------------------- */
			IfStatement(expr1, operator, expr2, body) => {
				let expr_type = infer_type(state, expr1)?;

				let value = eval_expression(state, expr1, &expr_type)?;
				let value2 = eval_expression(state, expr2, &expr_type)?;

				let accumulator = get_accumulator(&expr_type.word);

				if (value != accumulator) {
					state.textsect.push_str(&format!("\tmov {accumulator}, {} {value}\n", expr_type.word));
				}
				
				state.textsect.push_str(&format!("\tcmp {accumulator}, {value2}\n"));
				
				let jump_instruction = match operator {
					ComparisonOperator::Equal => "jne",
					ComparisonOperator::NotEqual => "je",

					ComparisonOperator::GreaterThan => "jle",
					ComparisonOperator::LessThan => "jge",

					ComparisonOperator::GreaterThanEqual => "jl",
					ComparisonOperator::LessThanEqual => "jg",
				};

				let old_labels = state.labels;

				state.labels += 1;
				state.textsect.push_str(&format!("\t{jump_instruction} .L{}\n", state.labels));
				generate(state, body)?;
				
				state.textsect.push_str(&format!(".L{}:\n", old_labels+1));
			},
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
			VariableDefinition(name, vartype, initexpr) => {
				let vartype = match vartype {
					Some(x) => DataType::new(x, state.line)?,
					None => infer_type(state, &initexpr.clone().unwrap())? /* unwrap will never fail */ 
				};

				if let Some(initexpr) = initexpr {
					let value = eval_expression(state, initexpr, &vartype)?;
					add_variable(state, name, &vartype, Some(&value))?;
				}
				else {
					add_variable(state, name, &vartype, None)?;
				}
			},
			/* -------------------------*/ 
			/*    variable assignment   */ 
			/* -------------------------*/ 
			VariableAssigment(name, expr) => {
				let variable = match state.current_function.local_variables.get(name).cloned() {
					Some(x) => x,
					None => return Err((format!("attempted to assign a value to variable '{name}', but it is not defined in the current scope"), state.line))
				};

				let value = eval_expression(state, expr, &variable.vartype)?;
				state.textsect.push_str(&format!("\tmov {} {}, {value}\n", variable.vartype.word, variable.addr));
			}
			/* -------------------------- */
			/*           macros           */
			/* -------------------------- */
			MacroCall(name, args) => {
				macros::call_macro(state, name, args)?;
			},
		}
	}

	Ok(())
}
