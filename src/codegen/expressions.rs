use super::*;

/* evaluates an expression (aka a list of tokens) and returns the result where its stored at (or just its literal value if no operations were done on it) */
/* an example input of [5, +, 5, +, strlen, (, "12345", )] with expected_type as i32 would give you the result 'r11d', which is where the result of this expression is stored at (the result is 15 by the way) */
/* another example input of ["hello world"] would return the identifier for this string literal, so something like L0 or L1 */
/* another example input of [5] would return 5 */
pub fn eval_expression(state: &mut State, expr: &Expression, expected_type: &DataType) -> Result<String, (String, i64)> {
	let mut iter = expr.iter();

	/* what this function does is it evaluates a single element of an expression, a sort of "miniexpression" */
	/* so if you feed it '5' it will return '5' */
	/* if you feed it the string literal "hello" it will return L0, L1, etc */
	/* if you feed it 'strlen, (, "hi", )' it will return rax/eax */
	/* if you feed it 'var' it will return its address on the stack (like [rbp-16]) */
	fn eval_miniexpression(state: &mut State, iter: &mut core::slice::Iter<Token>, expected_type: &DataType) -> Result<String, (String, i64)> {
		Ok(match (iter.next(), iter.clone().peekable().peek()) {
			(Some(Numerical(x)), _) => x.to_string(),
			(Some(StringLiteral(x)), _) => resolve_string_literal(&mut state.datasect, x),

			/* function calls */
			(Some(Identifier(name)), Some(Operator(LeftParen))) if !name.ends_with('!') => {
				iter.next(); /* strip ( */
				let args = process_function_parameters(iter);
				
				call_function(state, name, &args)?;

				/* unwrap will never fail as call_funtion will have hanndled it at this point */
				let function = state.functions.get(name).unwrap();

				let return_type = match &function.return_type {
					Some(x) => x.clone(),
					None => return Err((format!("attempted to get return value of function '{name}', but it does not return anything"), state.line))
				};

				if (&return_type != expected_type) {
					return Err((format!("expected expression to evaluate to '{}', but the return type of '{name}' is '{}'", expected_type.string, return_type.string), state.line));
				}

				get_accumulator(&return_type.word).to_owned()
			}

			/* macro calls */
			(Some(Identifier(name)), Some(Operator(LeftParen))) if name.ends_with('!') => {
				iter.next(); /* strip ( */
				let args = process_function_parameters(iter);
				
				let macro_obj = macros::get_macro(state, name)?;

				match macro_obj.return_type {
					Some(x) => DataType::new(x, state.line)?,
					None => return Err((format!("attempted to get return value of macro '{name}', but it does not return anything"), state.line))
				};

				/* call the macro and return its return value */
				/* expect will only fail if we set up the macro wrong */
				(macro_obj.function)(state, &args)?.unwrap_or_else(|| panic!("macro {name} returns a value of type '{:?}', but when calling it, it did not return a value", macro_obj.return_type))
			}

			/* variables */
			(Some(Identifier(x)), _) => {
				/* we move the variable to a temporary register and then pass that into add_variable */
				/* we have to use a temp register because we can't mov a memory location to another memory location obv */
				let var = match state.function.local_variables.get(x) {
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
			Operator(Plus) => {
				state.textsect.push_str(&format!("\tadd {root_register}, {val}\n"));
			},
			Operator(Dash) => {
				state.textsect.push_str(&format!("\tsub {root_register}, {val}\n"));
			}
			Operator(Star) => {
				state.textsect.push_str(&format!("\timul {root_register}, {val}\n"));
			}
			Operator(Slash) => {
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

/* infers a type from an expression */
pub fn infer_type(state: &mut State, expr: &Expression) -> Result<DataType, (String, i64)> {
	let mut iter = expr.iter();
	match iter.next() {
		Some(Identifier(identifier)) => {
			/* function/macro calls */
			if let Some(Operator(LeftParen)) = iter.next() {
				/* return function return type */
				if (!identifier.ends_with('!')) {
					match state.functions.get(identifier) {
						Some(x) => match &x.return_type {
							Some(x) => return Ok(x.clone()), /* we return here */
							None => return Err((format!("attempted to use return value of function '{identifier}' in expression but it does not return anything"), state.line))
						},
						None => return Err((format!("attempted to call function '{identifier}' in expression but it is not defined in the current scope"), state.line))
					}
				}

				/* return macro return type */
				match macros::get_macro(state, identifier)?.return_type {
					Some(x) => Ok(DataType::new(x, state.line)?), /* we return here */
					None => Err((format!("attempted to use return value of macro '{identifier}' in expression but it does not return anything"), state.line))
				}
			}
			/* variables */
			else {
				match state.function.local_variables.get(identifier) {
					Some(x) => Ok(x.vartype.clone()),
					None => Err((format!("attempted to use variable '{identifier}' in expression but it is not defined in the current scope"), state.line))
				}
			}
		}
		Some(Numerical(_)) => DataType::new("i32", state.line),
		Some(StringLiteral(_)) => DataType::new("i64", state.line),
		
		Some(err) => Err((format!("expected an identifier, int literal, or string literal as the first element of expression, but got {err} instead"), state.line)),
		None => Err((String::from("expected an identifier, int literal, or string literal as the first element of expression, but got nothing"), state.line))
	}
}