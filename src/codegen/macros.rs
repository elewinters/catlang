use super::*;

type MacroDefinition = fn(&mut State, &Vec<Expression>) -> Result<(), (String, i64)>;

pub const MACROS: [(&str, MacroDefinition); 2] = [
	("asm!", asm as MacroDefinition),
	("syscall!", syscall as MacroDefinition)
];

/* -------------- */
/*      asm!      */
/* -------------- */
pub fn asm(state: &mut State, args: &Vec<Expression>) -> Result<(), (String, i64)> {
	let instruction = match &args[0][0] {
		StringLiteral(ref x) => x,
		err => return Err((format!("expected token type to be a string literal, not {err}"), state.line))
	};

	state.textsect.push_str(&format!("\t{}\n", instruction));

	Ok(())
}

/* ------------------ */
/*      syscall!      */
/* ------------------ */
pub fn syscall(state: &mut State, args: &Vec<Expression>) -> Result<(), (String, i64)> {
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