use crate::lexer;
use super::*;

type MacroDefinition = fn(&mut State, &[Expression]) -> Result<Option<String>, (String, i64)>;

pub struct Macro {
	pub return_type: Option<&'static str>,
	pub function: MacroDefinition 
}

const MACROS: [(&str, Macro); 3] = [
	("asm!", Macro {
		return_type: None,
		function: asm
	}),

	("syscall!", Macro {
		return_type: Some("i64"),
		function: syscall
	}),

	("typeof!", Macro {
		return_type: Some("i64"), 
		function: typeof_}
	)
];

pub fn get_macro(state: &State, macro_name: &str) -> Result<Macro, (String, i64)> {
	for (name, macro_obj) in MACROS {
		if (name == macro_name) {
			return Ok(macro_obj);
		}
	}

	Err((format!("macro '{}' does not exist", macro_name), state.line))
}

pub fn call_macro(state: &mut State, macro_name: &str, args: &[Expression]) -> Result<Option<String>, (String, i64)> {
	let function_ptr = get_macro(state, macro_name)?.function;
	function_ptr(state, args)
}

/* ----------------- */
/*      typeof!      */
/* ----------------- */
fn typeof_(state: &mut State, args: &[Expression]) -> Result<Option<String>, (String, i64)> {
	if (args.len() != 1) {
		return Err((format!("typeof! macro accepts 1 argument, not {}", args.len()), state.line))
	}

	let variable = match &args[0][0] {
		Identifier(x) => match state.current_function.local_variables.get(x) {
			Some(x) => x,
			None => return Err((format!("variable '{x}' is not defined in the current scope in call to typeof! macro"), state.line))
		}
		err => return Err((format!("argument to typeof! must be a valid identifier, not {err}"), state.line))
	};

	let to_return = resolve_string_literal(&mut state.datasect, &variable.vartype.string);

	Ok(Some(to_return))
}

/* -------------- */
/*      asm!      */
/* -------------- */
fn parse_asm(state: &State, input: &str) -> Result<String, (String, i64)> {
	let tokens = match lexer::lex(input) {
		Ok(x) => x,
		Err(err) => return Err((err, state.line))
	};

	let mut iter = tokens.iter();

	let mut output = String::new();

	while let Some(i) = iter.next() {
		match (i) {
			Operator(LeftCurly) => {
				let identifier = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err((String::from("expected identifier after operator '{' in asm! macro call"), state.line))
				};
				
				let variable = match state.current_function.local_variables.get(identifier) {
					Some(x) => x,
					None => return Err((format!("undeclared variable '{identifier}' in asm! macro call"), state.line))
				};

				match iter.next() {
					Some(Operator(RightCurly)) => (),
					_ => return Err((format!("expected operator '}}' after identifier '{identifier}' in asm! macro call"), state.line))
				}

				output.push_str(&format!("{} ", variable.addr));
			}

			Keyword(x) => output.push_str(&format!("{:?}", x).to_lowercase()),
			
			Identifier(x) | Numerical(x) => output.push_str(&format!("{x} ")),
			Operator(Comma) => { output.pop(); output.push_str(", ") },
			Operator(_) => todo!("operators in asm! macro"),

			StringLiteral(_) => return Err((String::from("string literals are not allowed in the asm! macro"), state.line)),
			Token::Newline => output.push_str("\n\t"),
		}	
	}

	output = output.trim().to_owned();

	Ok(output)
}

fn asm(state: &mut State, args: &[Expression]) -> Result<Option<String>, (String, i64)> {
	if (args.len() != 1) {
		return Err((format!("asm! macro accepts 1 argument, not {}", args.len()), state.line))
	}

	let instruction = match &args[0][0] {
		StringLiteral(ref x) => x,
		err => return Err((format!("expected token type to be a string literal, not {err}"), state.line))
	};

	/* state doesnt get mutated here, just read  */
	let parsed = parse_asm(state, instruction)?;
	state.textsect.push_str(&format!("\t{parsed}\n\n"));

	Ok(None)
}

/* ------------------ */
/*      syscall!      */
/* ------------------ */
fn syscall(state: &mut State, args: &[Expression]) -> Result<Option<String>, (String, i64)> {
	for (i, v) in args.iter().enumerate() {
		let v = eval_expression(state, v, &DataType::new("i64", state.line)?)?;

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

	Ok(Some(String::from("rax")))
}