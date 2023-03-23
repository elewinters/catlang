use std::collections::HashMap;

mod registers;
use registers::*;

mod macros;

use crate::parser::AstType::{self, *};
use crate::lexer::TokenType::{self, *};

use crate::parser::{process_function_parameters, Expression, ComparisonOperator};

/* ------------------------------ */
/*           structures           */
/* ------------------------------ */

#[derive(Clone, PartialEq)]
struct DataType {
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
	local_variables: HashMap<String, Variable>,
	return_type: Option<DataType>,
	stacksize: i32,
	stackspace: i32,

	calls_funcs: bool,
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
			local_variables: HashMap::new(),
			return_type: None,
			
			/* stacksize needs to start at 8 because whenever we push rbx, [rbp-8] becomes the location of rbx */
			/* this is bad because whenever we make a variable we will start at [rbp-4] or [rbp-8] */
			/* meaning we will overwrite rbx on the stack */
			/* took me a bit to figure this out */
			stacksize: 8,
			stackspace: 0,

			calls_funcs: false
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

/* evaluates an expression (aka a list of tokens) and returns the result where its stored at (or just its literal value if no operations were done on it) */
/* an example input of [5, +, 5, +, strlen, (, "12345", )] with expected_type as i32 would give you the result 'r11d', which is where the result of this expression is stored at (the result is 15 by the way) */
/* another example input of ["hello world"] would return the identifier for this string literal, so something like L0 or L1 */
/* another example input of [5] would return 5 */
fn eval_expression(state: &mut State, expr: &Expression, expected_type: &DataType) -> Result<String, (String, i64)> {
	let mut iter = expr.iter();

	/* what this function does is it evaluates a single element of an expression, a sort of "miniexpression" */
	/* so if you feed it '5' it will return '5' */
	/* if you feed it the string literal "hello" it will return L0, L1, etc */
	/* if you feed it 'strlen, (, "hi", )' it will return rax/eax */
	/* if you feed it 'var' it will return its address on the stack (like [rbp-16]) */
	fn eval_miniexpression(state: &mut State, iter: &mut core::slice::Iter<TokenType>, expected_type: &DataType) -> Result<String, (String, i64)> {
		Ok(match (iter.next(), iter.clone().peekable().peek()) {
			(Some(IntLiteral(x)), _) => x.to_string(),
			(Some(StringLiteral(x)), _) => resolve_string_literal(&mut state.datasect, x),

			/* function/macro calls */
			(Some(Identifier(name)), Some(Operator('('))) => {
				iter.next(); /* strip ( */
				let args = process_function_parameters(iter);
				
				let return_type = if (!name.ends_with('!')) {
					call_function(state, name, &args)?;

					/* unwrap will never fail as call_funtion will have hanndled it at this point */
					let function = state.functions.get(name).unwrap();
	
					match &function.return_type {
						Some(x) => x.clone(),
						None => return Err((format!("attempted to get return value of function '{name}', but it does not return anything"), state.line))
					}
				}
				else {
					macros::call_macro(state, name, &args)?;
					let macro_obj = macros::get_macro(state, name)?;

					match macro_obj.return_type {
						Some(x) => DataType::new(x, state.line)?,
						None => return Err((format!("attempted to get return value of macro '{name}', but it does not return anything"), state.line))
					}
				};
		
				if (&return_type != expected_type) {
					return Err((format!("expected expression to evaluate to '{}', but the return type of '{name}' is '{}'", expected_type.string, return_type.string), state.line));
				}

				get_accumulator(&return_type.word).to_owned()
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
				if (&var.vartype != expected_type) {
					return Err((format!("expected expression to evaluate to type '{}', but the type of '{x}' is '{}'", expected_type.string, var.vartype.string), state.line));
				}
	
				var.addr.clone()
			}
			
			(Some(x), _) => return Err((format!("expected int literal, string literal or identifier in expression, but got {x}"), state.line)),
			(None, _) => return Err((String::from("expected int literal, string literal, or identifier in expression, but got nothing"), state.line))
		})
	}

	let root_register = get_rbx(&expected_type.word);
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
				let accumulator = get_accumulator(&expected_type.word);
				let r11 = get_r11(&expected_type.word);

				/* clear out rdx before division (if we dont do this we will Crash the Fucking Program) */
				state.textsect.push_str("\n\tcdq\n");
				
				state.textsect.push_str(&format!("\tmov {r11}, {val}\n"));
				state.textsect.push_str(&format!("\tmov {accumulator}, {root_register}\n"));
				state.textsect.push_str(&format!("\tidiv {r11}\n"));

				state.textsect.push_str(&format!("\tmov {root_register}, {accumulator}\n\n"));
			}

			err => return Err((format!("unexpected {err} in expression evaluation"), state.line))
		}
	}

	/* once we're done we return it */
	Ok(root_register.to_owned())
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

/* infers a type from an expression */
fn infer_type(state: &mut State, expr: &Expression) -> Result<DataType, (String, i64)> {
	let mut iter = expr.iter();
	match iter.next() {
		Some(TokenType::Identifier(identifier)) => {
			/* function/macro calls */
			if let Some(Operator('(')) = iter.next() {
				if (!identifier.ends_with('!')) {
					match state.functions.get(identifier) {
						Some(x) => match &x.return_type {
							Some(x) => return Ok(x.clone()), /* we return here */
							None => return Err((format!("attempted to use return value of function '{identifier}' in expression but it does not return anything"), state.line))
						},
						None => return Err((format!("attempted to call function '{identifier}' in expression but it is not defined in the current scope"), state.line))
					}
				}

				match macros::get_macro(state, identifier)?.return_type {
					Some(x) => Ok(DataType::new(x, state.line)?), /* we return here */
					None => Err((format!("attempted to use return value of macro '{identifier}' in expression but it does not return anything"), state.line))
				}
			}
			/* variables */
			else {
				match state.current_function.local_variables.get(identifier) {
					Some(x) => Ok(x.vartype.clone()),
					None => Err((format!("attempted to use variable '{identifier}' in expression but it is not defined in the current scope"), state.line))
				}
			}
		}
		Some(TokenType::IntLiteral(_)) => DataType::new("i32", state.line),
		Some(TokenType::StringLiteral(_)) => DataType::new("i64", state.line),
		
		Some(err) => Err((format!("expected an identifier, int literal, or string literal as the first element of expression, but got {err} instead"), state.line)),
		None => Err((String::from("expected an identifier, int literal, or string literal as the first element of expression, but got nothing"), state.line))
	}
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

				/* parse and append the body of the funnction */
				generate(state, body)?;

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
