use std::fmt::{self, Display};
use self::WordType::*;

#[derive(Clone, PartialEq)]
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

pub fn get_accumulator(word: &WordType) -> &'static str {
	match word {
		Byte => "al",
		Word => "ax",
		DoubleWord => "eax",
		QuadWord => "rax",
	}
}

pub fn get_rbx(word: &WordType) -> &'static str {
	match word {
		Byte => "bl",
		Word => "bx",
		DoubleWord => "ebx",
		QuadWord => "rbx"
	}
}

pub fn get_r11(word: &WordType) -> &'static str {
	match word {
		Byte => "r11b",
		Word => "r11w",
		DoubleWord => "r11d",
		QuadWord => "r11"
	}
}

pub fn get_register(argument_count: usize, word: &WordType) -> &'static str {
    match (argument_count, word) {
        /* edi/rdi */
        (0, Byte) => "dil",
        (0, Word) => "di", 
        (0, DoubleWord) => "edi",
        (0, QuadWord) => "rdi",

        /* esi/rsi */
        (1, Byte) => "sil", 
        (1, Word) => "si",
        (1, DoubleWord) => "esi",
        (1, QuadWord) => "rsi",

        /* edx/rdx */
        (2, Byte) => "dl",
        (2, Word) => "dx",
        (2, DoubleWord) => "edx",
        (2, QuadWord) => "rdx",

        /* ecx/rcx */
        (3, Byte) => "cl",
        (3, Word) => "cx",
        (3, DoubleWord) => "ecx",
        (3, QuadWord) => "rcx",
        
        /* r8 */
        (4, Byte) => "r8b",
        (4, Word) => "r8w",
        (4, DoubleWord) => "r8d",
        (4, QuadWord) => "r8",

        /* r9 */
        (5, Byte) => "r9b",
        (5, Word) => "r9w",
        (5, DoubleWord) => "r9d",
        (5, QuadWord) => "r9",

        (x, _) => panic!("called get_register in an attempt to get the register for the {0}nd argument, this should never happen as a function that has more than 6 arguments should push them on the stack", x+1)
    }
}