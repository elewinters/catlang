pub fn get_size_of_type(input: &str, line: i64) -> Result<(&'static str, i32), (String, i64)> {
	match (input) {
		"i8" => Ok(("byte", 1)),
		"i16" => Ok(("word", 2)),
		"i32" => Ok(("dword", 4)),
		"i64" => Ok(("qword", 8)),
		_ => return Err((format!("'{input}' is not a valid type"), line))
	}
} 

pub fn get_register_call(argument_count: usize, argword: &str, line: i64) -> Result<&'static str, (String, i64)> {
    match (argument_count, argword) {
        /* edi/rdi */
        (0, "byte") | (0, "word") | (0, "dword") => Ok("edi"),
        (0, "qword") => Ok("rdi"),

        /* esi/rsi */
        (1, "byte") | (1, "word") | (1, "dword") => Ok("esi"),
        (1, "qword") => Ok("rsi"),

        /* edx/rdx */
        (2, "byte") | (2, "word") | (2, "dword") => Ok("edx"),
        (2, "qword") => Ok("rdx"),

        /* ecx/rcx */
        (3, "byte") | (3, "word") | (3, "dword") => Ok("ecx"),
        (3, "qword") => Ok("rcx"),
        
        /* r8 */
        (4, "byte") | (4, "word") |(4, "dword") => Ok("r8d"),
        (4, "qword") => Ok("r8"),

        /* r9 */
        (5, "byte") | (5, "word") | (5, "dword") => Ok("r9d"),
        (5, "qword") => Ok("r9"),

        (c, t) => {
            if (c > 5) {
                Err((String::from("too many arguments to function, functions can only up to 6 arguments at the moment"), line))
            }
            else {
                Err((format!("'{t}' is not a valid word"), line))
            }
        }
    }
}

pub fn get_register_definition(argument_count: usize, argword: &str, line: i64) -> Result<&'static str, (String, i64)> {
    match (argument_count, argword) {
        /* edi/rdi */
        (0, "byte") => Ok("dil"),
        (0, "word") => Ok("di"), 
        (0, "dword") => Ok("edi"),
        (0, "qword") => Ok("rdi"),

        /* esi/rsi */
        (1, "byte") => Ok("sil"), 
        (1, "word") => Ok("si"),
        (1, "dword") => Ok("esi"),
        (1, "qword") => Ok("rsi"),

        /* edx/rdx */
        (2, "byte") => Ok("dl"),
        (2, "word") => Ok("dx"),
        (2, "dword") => Ok("edx"),
        (2, "qword") => Ok("rdx"),

        /* ecx/rcx */
        (3, "byte") => Ok("cl"),
        (3, "word") => Ok("cx"),
        (3, "dword") => Ok("ecx"),
        (3, "qword") => Ok("rcx"),
        
        /* r8 */
        (4, "byte") => Ok("r8b"),
        (4, "word") => Ok("r8w"),
        (4, "dword") => Ok("r8d"),
        (4, "qword") => Ok("r8"),

        /* r9 */
        (5, "byte") => Ok("r9b"),
        (5, "word") => Ok("r9w"),
        (5, "dword") => Ok("r9d"),
        (5, "qword") => Ok("r9"),

        (c, t) => {
            if (c > 5) {
                Err((String::from("too many arguments to function, functions can only up to 6 arguments at the moment"), line))
            }
            else {
                Err((format!("'{t}' is not a valid word"), line))
            }
        }
    }
}

pub fn get_accumulator(vartype: &str) -> &'static str {
	match vartype {
		"byte" => "al",
		"word" => "ax",
		"dword" => "eax",
		"qword" => "rax",

		_ => "eax"
	}
}