#![allow(unused_parens)]
use std::fs;
use std::process::Command;

mod options;
mod lexer;
mod ast;
mod parser;

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
	let assembly_output = parser::parse(&ast);

	if (!options.create_binary) {
		let result = match (options.output_name) {
			Some(x) => fs::write(x, &assembly_output),
			None => fs::write("output.asm", &assembly_output)
		};

		if let Err(err) = result {
			println!("catlang: \x1b[31merror:\x1b[0m failed to write assembly output, {}", err);
			std::process::exit(1);
		}
	}
	/* assemble and link output to create a binary */
	else {
		if fs::write("/tmp/catlang_output.asm", &assembly_output).is_err() {
			println!("catlang: \x1b[31merror:\x1b[0m failed to write assembly output to /tmp/catlang_output.asm");
			std::process::exit(1);
		}

		let output_name = options.output_name.unwrap_or(String::from("output")); 

		Command::new("nasm")
		.arg("-felf64")
		.arg("/tmp/catlang_output.asm")
		.arg("-o")
		.arg("/tmp/catlang_output.o")
		.spawn()
		.unwrap()
		.wait()
		.unwrap();

		if (!options.link_libc) {
			Command::new("ld")
			.arg("/tmp/catlang_output.o")
			.arg("-o")
			.arg(&output_name)
			.spawn()
			.unwrap()
			.wait()
			.unwrap();
		}
		else {
			Command::new("gcc")
			.arg("-no-pie")
			.arg("/tmp/catlang_output.o")
			.arg("-o")
			.arg(&output_name)
			.spawn()
			.unwrap()
			.wait()
			.unwrap();
		}

		Command::new("rm")
		.arg("/tmp/catlang_output.asm")
		.arg("/tmp/catlang_output.o")
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
	}
}