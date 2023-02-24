#![allow(unused_parens)]
mod options;
mod lexer;
mod ast;

fn main() {
	let options = match options::get_options() {
		Ok(x) => x,
		Err(err) => {
			println!("catlang: \x1b[31moptions error:\x1b[0m {}", err);
			std::process::exit(1);
		}
	};
	if (options.verbose) {
		println!("{:?}", options);
	}

	/* lex now owns options.input */
	let tokens = lexer::lex(options.input);
	if (options.verbose) {
		lexer::print_tokens(&tokens);
	}

	/* generate AST */
	let ast = match ast::ast(&tokens) {
		Ok(x) => x,
		Err(err) => {
			println!("catlang: \x1b[31mparser error:\x1b[0m {}", err);
			std::process::exit(1);
		}
	};
}