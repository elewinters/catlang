#![allow(unused_assignments)]
use std::fs;
use std::env;

#[derive(Debug)]
pub struct Options {
	pub input: Vec<u8>,
	pub output_name: Option<String>,
	pub create_binary: bool,
	pub link_libc: bool,
	pub verbose: bool
}

fn print_help() {
	println!("catlang - a minimal compiled languge for the x86/x86_64 architecture");
	println!("usage: catlang [input_file] [options...]");
	println!("options: ");
	println!();
	println!("-h, --help - print this message");
	println!("-b, --create-binary - assemble and link output assembly to create an executable [must have nasm and ld installed]");
	println!("-lc, --link-libc - when creating a binary, automatically link libc with gcc [must have gcc installed]");
	println!("-V, --verbose - makes the compiler print some information about what it's doing");
	println!("-o, --output-name - set the filename of the output file/binary");
	
	std::process::exit(0);
}

pub fn get_options() -> Result<Options, String> {
	let args: Vec<String> = env::args().collect();
	if (args.len() == 1) {
		return Err(format!("no arguments specified, try '{}' --help' for more information", args[0]));
	}

	let mut options = Options {
		input: Vec::new(),
		output_name: None,
		create_binary: false,
		link_libc: false,
		verbose: false		
	};

	let mut i = 1;
	while i < args.len() {
		if (args[i] == "-h" || args[i] == "--help") {
			print_help();
		}
		else if (args[i] == "-b" || args[i] == "--create-binary") {
			options.create_binary = true;
		}
		else if (args[i] == "-lc" || args[i] == "--link-libc") {
			options.link_libc = true;
		}
		else if (args[i] == "-V" || args[i] == "--verbose") {
			options.verbose = true;
		}
		else if (args[i] == "-o" || args[i] == "--output-name") {
			if (args.len() <= i+1) {
				return Err(String::from("no output name specified after -o/--output-name option"));
			}
			
			options.output_name = Some(args[i+1].clone());

			i += 1;
		}
		else if (!args[i].starts_with('-')) {
			if (!options.input.is_empty()) {
				return Err(String::from("more than one input file"));
			}
			
			let input: Vec<u8> = match(fs::read(&args[i])) {
				Ok(x) => x,
				Err(_) => return Err(format!("input file '{}' cannot be read", args[i]))
			};

			if (!input.is_ascii()) {
				return Err(String::from("input file is not in ascii, please remove any unicode symbols"));
			}

			options.input = input;
		}
		else {
			return Err(format!("invalid option '{}'", args[i]));
		}

		i += 1;
	}

	if (options.link_libc && !options.create_binary) {
		return Err(String::from("--link_libc option is set but --create_binary option isn't, try removing it"));
	}

	if (options.input.is_empty()) {
		return Err(String::from("no input file"));
	}

	Ok(options)
}