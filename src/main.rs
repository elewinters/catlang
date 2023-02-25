#![allow(unused_parens)]
mod options;
mod lexer;
mod ast;

fn main() {
	/* get command line options */
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

	/* lex input string into tokens */
	let tokens = lexer::lex(options.input);
	if (options.verbose) {
		lexer::print_tokens(&tokens);
	}

	/* generate AST from tokens */
	let ast = match ast::ast(&tokens) {
		Ok(x) => x,
		Err((err, line)) => {
			println!("catlang: \x1b[31mparser error:\x1b[0m [line {line}] {}", err);
			std::process::exit(1);
		}
	};
	println!("\n\n{:?}", &ast);

	/* parse AST and generate the assembly code */
}