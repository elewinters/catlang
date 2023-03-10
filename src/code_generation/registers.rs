use std::fmt::{self, Display};
use self::WordType::*;

pub enum WordType {
	Byte,
	Word,
	DoubleWord,
	QuadWord
}

impl Display for WordType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Byte => write!(f, "byte"),
			Word => write!(f, "word"),
			DoubleWord => write!(f, "dword"),
			QuadWord => write!(f, "qword"),
		}
	}
}

pub fn get_size_of_type(input: &str, line: i64) -> Result<(WordType, i32), (String, i64)> {
	match (input) {
		"i8" => Ok((Byte, 1)),
		"i16" => Ok((Word, 2)),
		"i32" => Ok((DoubleWord, 4)),
		"i64" => Ok((QuadWord, 8)),
		_ => return Err((format!("'{input}' is not a valid type"), line))
	}
}

pub fn get_accumulator(word: &WordType) -> &'static str {
	match word {
		Byte => "al",
		Word => "ax",
		DoubleWord => "eax",
		QuadWord => "rax",
	}
}

pub fn get_accumulator2(word: &WordType) -> &'static str {
	match word {
		Byte => "r11b",
		Word => "r11w",
		DoubleWord => "r11d",
		QuadWord => "r11"
	}
}

pub fn get_register(argument_count: usize, word: &WordType, line: i64) -> Result<&'static str, (String, i64)> {
    match (argument_count, word) {
        /* edi/rdi */
        (0, Byte) => Ok("dil"),
        (0, Word) => Ok("di"), 
        (0, DoubleWord) => Ok("edi"),
        (0, QuadWord) => Ok("rdi"),

        /* esi/rsi */
        (1, Byte) => Ok("sil"), 
        (1, Word) => Ok("si"),
        (1, DoubleWord) => Ok("esi"),
        (1, QuadWord) => Ok("rsi"),

        /* edx/rdx */
        (2, Byte) => Ok("dl"),
        (2, Word) => Ok("dx"),
        (2, DoubleWord) => Ok("edx"),
        (2, QuadWord) => Ok("rdx"),

        /* ecx/rcx */
        (3, Byte) => Ok("cl"),
        (3, Word) => Ok("cx"),
        (3, DoubleWord) => Ok("ecx"),
        (3, QuadWord) => Ok("rcx"),
        
        /* r8 */
        (4, Byte) => Ok("r8b"),
        (4, Word) => Ok("r8w"),
        (4, DoubleWord) => Ok("r8d"),
        (4, QuadWord) => Ok("r8"),

        /* r9 */
        (5, Byte) => Ok("r9b"),
        (5, Word) => Ok("r9w"),
        (5, DoubleWord) => Ok("r9d"),
        (5, QuadWord) => Ok("r9"),

        (_, _) => {
            Err((String::from("too many arguments to function, functions can only up to 6 arguments at the moment"), line))
        }
    }
}

pub fn get_register_32_or_64(argument_count: usize, word: &WordType, line: i64) -> Result<&'static str, (String, i64)> {
    match (argument_count, word) {
        /* edi/rdi */
        (0, Byte) | (0, Word) | (0, DoubleWord) => Ok("edi"),
        (0, QuadWord) => Ok("rdi"),

        /* esi/rsi */
        (1, Byte) | (1, Word) | (1, DoubleWord) => Ok("esi"),
        (1, QuadWord) => Ok("rsi"),

        /* edx/rdx */
        (2, Byte) | (2, Word) | (2, DoubleWord) => Ok("edx"),
        (2, QuadWord) => Ok("rdx"),

        /* ecx/rcx */
        (3, Byte) | (3, Word) | (3, DoubleWord) => Ok("ecx"),
        (3, QuadWord) => Ok("rcx"),
        
        /* r8 */
        (4, Byte) | (4, Word) |(4, DoubleWord) => Ok("r8d"),
        (4, QuadWord) => Ok("r8"),

        /* r9 */
        (5, Byte) | (5, Word) | (5, DoubleWord) => Ok("r9d"),
        (5, QuadWord) => Ok("r9"),

        (_, _) => {
            Err((String::from("too many arguments to function, functions can only up to 6 arguments at the moment"), line))
        }
    }
}