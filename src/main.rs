#![allow(unused_parens)]
use std::fs;
use std::env;
use std::process::Command;

mod options;
mod lexer;
mod parser;
mod code_generation;

macro_rules! exit {
	($fmt:expr) => {
		{
			println!("{}", String::from("catlang: \x1b[31merror:\x1b[0m ") + &$fmt);
			std::process::exit(1);
		}
	}
}

/* 
	TODO:
		rewrite the lexer
		add a nicer way to print the AST
		make expressions more versatile

		NOW:
			replace String with enums for all of the datatypes and word types

			add control flow (if & match)
			add arrays
			add support for variadic functions

			add loops (while & for)
			
*/

fn main() {
	/* ---------------------------- */
	/*   get command line options   */
	/* ---------------------------- */
	let options = match options::get_options() {
		Ok(x) => x,
		Err(err) => exit!(format!("{}", err))
	};
	if (options.verbose) {
		println!("---------------------");
		println!("       options       ");
		println!("---------------------");
		println!("{:?}", options);
	}

	/* --------------------------- */
	/*    lex input into tokens    */
	/* --------------------------- */
	let tokens = lexer::lex(options.input);
	if (options.verbose) {
		println!("--------------------");
		println!("       tokens       ");
		println!("--------------------");
		lexer::print_tokens(&tokens);
	}

	/* --------------------------- */
	/*   generate AST from tokens  */
	/* --------------------------- */
	let ast = match parser::parse(&tokens) {
		Ok(x) => x,
		Err((err, line)) => exit!(format!("[line {line}] {}", err))
	};
	if (options.verbose) {
		println!("------------------------");
		println!("  abstract syntax tree  ");
		println!("------------------------");
		println!("{:?}", &ast);
	}

	/* -------------------------------------------- */
	/*  parse AST and generate the assembly output  */
	/* -------------------------------------------- */
	let assembly_output = match code_generation::generate(&ast) {
		Ok(x) => x,
		Err((err, line)) => exit!(format!("[line {line}] {}", err))
	};

	/* --------------------------------- */
	/*  write assembly output to a file  */
	/* --------------------------------- */
	if (!options.create_binary) {
		let result = match (options.output_name) {
			Some(x) => fs::write(x, &assembly_output),
			None => fs::write("output.asm", &assembly_output)
		};

		if let Err(err) = result {
			exit!(format!("failed to write assembly output, {}", err));
		}
	}
	/* ------------------------------------------------------ */
	/*  assemble and link assembly output to create a binary  */
	/* ------------------------------------------------------ */
	else {
		let temp_dir = env::temp_dir();
		let temp = temp_dir.display();

		if fs::write(format!("{temp}/catlang_output.asm"), &assembly_output).is_err() {
			exit!(format!("failed to write assembly output to {temp}/catlang_output.asm"));
		}

		let output_name = options.output_name.unwrap_or(String::from("output")); 

		Command::new("nasm")
			.arg("-felf64")
			.arg(format!("{temp}/catlang_output.asm"))
			.arg("-o")
			.arg(format!("{temp}/catlang_output.o"))
			.spawn()
			.unwrap()
			.wait()
			.unwrap();

		if (!options.link_libc) {
			Command::new("ld")
				.arg(format!("{temp}/catlang_output.o"))
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
				.arg(format!("{temp}/catlang_output.o"))
				.arg("-o")
				.arg(&output_name)
				.spawn()
				.unwrap()
				.wait()
				.unwrap();
		}

		if let Err(err) = fs::remove_file(format!("{temp}/catlang_output.asm")) {
			exit!(format!("failed to delete {temp}/catlang_output.asm, {err}"))
		}

		if let Err(err) = fs::remove_file(format!("{temp}/catlang_output.o")) {
			exit!(format!("failed to delete {temp}/catlang_output.o, {err}"))
		}
	}
}