use std::fs;
use std::env;

#[derive(Debug, Default)]
pub struct Options {
	pub input: String,
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
	let mut args = env::args();
	let mut options = Options::default();

	if (args.len() == 1) {
		return Err(String::from("no arguments specified, try 'catlang --help' for more information"));
	}
	
	/* get rid of the program name */
	args.next();

	while let Some(i) = args.next() {
		if (i == "-h" || i == "--help") {
			print_help();
		}
		else if (i == "-b" || i == "--create-binary") {
			options.create_binary = true;
		}
		else if (i == "-lc" || i == "--link-libc") {
			options.link_libc = true;
		}
		else if (i == "-V" || i == "--verbose") {
			options.verbose = true;
		}
		else if (i == "-o" || i == "--output-name") {
			options.output_name = args.next();
		}
		else if (!i.starts_with('-')) {
			if (!options.input.is_empty()) {
				return Err(String::from("more than one input file"));
			}
			
			let input = match(fs::read_to_string(&i)) {
				Ok(x) => x,
				Err(err) => return Err(format!("input file '{}' cannot be read [{err}]", i))
			};

			if (!input.is_ascii()) {
				return Err(String::from("input file is not in ascii, please remove any unicode symbols"));
			}

			options.input = input;
		}
		else {
			return Err(format!("invalid option '{}'", i));
		}
	}

	if (options.link_libc && !options.create_binary) {
		return Err(String::from("--link_libc option is set but --create_binary option isn't"));
	}

	if (options.input.is_empty()) {
		return Err(String::from("no input file"));
	}

	Ok(options)
}