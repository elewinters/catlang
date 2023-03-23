use crate::lexer;
use super::*;

type MacroDefinition = fn(&mut State, &[Expression]) -> Result<(), (String, i64)>;

const MACROS: [(&str, MacroDefinition); 2] = [
	("asm!", asm as MacroDefinition),
	("syscall!", syscall as MacroDefinition)
];

pub fn call_macro(state: &mut State, macro_name: &str, args: &[Expression]) -> Result<(), (String, i64)> {
	for (name, function) in macros::MACROS {
		if (name == macro_name) {
			function(state, args)?;
			return Ok(());
		}
	}

	Err((format!("macro '{}' does not exist", macro_name), state.line))
}

/* -------------- */
/*      asm!      */
/* -------------- */
fn parse_asm(state: &State, input: &str) -> Result<String, (String, i64)> {
	let tokens = lexer::lex(input.as_bytes());
	let mut iter = tokens.iter();

	let mut output = String::new();

	while let Some(i) = iter.next() {
		match (i) {
			Operator('{') => {
				let identifier = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err((String::from("expected identifier after operator '{' in asm! macro call"), state.line))
				};
				
				let variable = match state.current_function.local_variables.get(identifier) {
					Some(x) => x,
					None => return Err((format!("undeclared variable '{identifier}' in asm! macro call"), state.line))
				};

				match iter.next() {
					Some(Operator('}')) => (),
					_ => return Err((format!("expected operator '}}' after identifier '{identifier}' in asm! macro call"), state.line))
				}

				output.push_str(&format!(" {}", variable.addr));
			}

			Keyword(x) | Identifier(x) | IntLiteral(x) => output.push_str(&format!(" {x}")),
			Operator(x) => output.push(*x),

			StringLiteral(_) => return Err((String::from("string literals are not allowed in the asm! macro"), state.line)),
			TokenType::Newline => panic!("newline token in asm! macro call, this is never supposed to happen")
		}	
	}

	/* strip the whitespace space at the beginning of the string */
	output.remove(0);

	Ok(output)
}

fn asm(state: &mut State, args: &[Expression]) -> Result<(), (String, i64)> {
	let instruction = match &args[0][0] {
		StringLiteral(ref x) => x,
		err => return Err((format!("expected token type to be a string literal, not {err}"), state.line))
	};

	/* state doesnt get mutated here, just read  */
	let parsed = parse_asm(state, instruction)?;
	state.textsect.push_str(&format!("\t{parsed}\n"));

	Ok(())
}

/* ------------------ */
/*      syscall!      */
/* ------------------ */
fn syscall(state: &mut State, args: &[Expression]) -> Result<(), (String, i64)> {
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

	Ok(())
}