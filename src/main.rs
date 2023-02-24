#![allow(unused_parens)]

mod lexer;
mod options;

fn main() {
	let options = match (options::get_options()) {
		Ok(x) => x,
		Err(err) => {
			println!("catlang: \x1b[31moptions error:\x1b[0m {}", err);
			std::process::exit(1);
		}
	};
	println!("{:?}", options);

	/* lex now owns options.input */
	let tokens = lexer::lex(options.input);
	lexer::print_tokens(&tokens);
}