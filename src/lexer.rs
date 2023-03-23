use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
	Keyword(String),
	Identifier(String),

	StringLiteral(String),
	IntLiteral(String),

	Operator(char),
	Newline
}

impl Display for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TokenType::Keyword(x) => write!(f, "keyword '{x}'"),
			TokenType::Identifier(x) => write!(f, "identifier '{x}'"),

			TokenType::StringLiteral(x) => write!(f, "string literal '{x}'"),
			TokenType::IntLiteral(x) => write!(f, "int literal '{x}'"),

			TokenType::Operator(x) => write!(f, "operator '{x}'"),
			TokenType::Newline => write!(f, "newline")
		}
	}
}

const KEYWORDS: [&str; 4] = [
	"let", 
	"fn",
	"return",
	"if"
];

const ESCAPE_CHARACTERS: [char; 14] = [
	'<', 
	'>', /* for the arrows in function returns (like -> i32) */
	'+',
	'-',
	'*',
	'/',
	'(', 
	')', 
	',', 
	'=', 
	';',
	':',
	'{', 
	'}'
];

pub fn print_tokens(input: &Vec<TokenType>) {
	for i in input {
		match (i) {
			TokenType::Keyword(x) => print!("Keyword[{}], ", x),
			TokenType::Identifier(x) => print!("Identifier[{}], ", x),
			TokenType::StringLiteral(x) => print!("StringLiteral[\"{}\"], ", x),
			TokenType::IntLiteral(x) => print!("IntLiteral[{}], ", x),

			TokenType::Operator(x) => print!("Operator[{}], ", x),

			TokenType::Newline => println!("Newline")
		}
	}
	println!();
}

pub fn lex(input: &[u8]) -> Vec<TokenType> {
	let mut tokens: Vec<TokenType> = Vec::new();
	let mut iter = input.iter().peekable();

	let mut in_comment = false;

	while let Some(v) = iter.next() {
		/* comment handling */
		{
			if (*v == b'/') {
				if let Some(b'*') = iter.peek() {
					in_comment = true;
					iter.next();
				}
			}

			if (*v == b'*') {
				if let Some(b'/') = iter.peek() {
					in_comment = false;
					iter.next();

					continue;
				}
			}

			if (in_comment) {
				continue;
			}
		}	

		if (*v == b'\n') {
			tokens.push(TokenType::Newline);
		}
		
		if (v.is_ascii_whitespace()) {
			continue;
		}

		if (v.is_ascii_punctuation() && *v != b'"' && *v != b'_') {
			tokens.push(TokenType::Operator(*v as char));
			continue; /* not doing this caused whitespace to mess stuff up sometimes */
		}
		
		let mut identifier = String::new();
		/* iter.next() below immediately skips, so we have */
		/* to add this before that happens */
		identifier.push(*v as char);
		
		let mut escaped: char = '\0';
		let mut is_string: bool = false;
		for (i, c) in iter.by_ref().enumerate() {
			if (*v == b'"') {
				is_string = true;
			}

			if ((*c as char).is_whitespace() && !is_string) {
				if (*c as char) == '\n' {
					escaped = '\n';
				}
				break;
			}
			if (ESCAPE_CHARACTERS.contains(&(*c as char)) && !is_string) {
				escaped = (*c as char);
				break;
			}

			if (i != 0 && (*c as char) == '"') {
				break;
			}

			identifier.push(*c as char);
		}
		
		/* if its a keyword we change the type to Keyword */
		if (KEYWORDS.contains(&identifier.as_ref())) {
			tokens.push(TokenType::Keyword(identifier));
		}
		else if (v.is_ascii_alphabetic() || *v == b'_') {
			tokens.push(TokenType::Identifier(identifier));
		}
		else if (v.is_ascii_alphanumeric()) {
			tokens.push(TokenType::IntLiteral(identifier));
		}
		else if (*v == b'"') {
			identifier.remove(0);
			tokens.push(TokenType::StringLiteral(identifier));
		}

		if (escaped == '\n') {
			tokens.push(TokenType::Newline);
			continue;
		}
		
		if (!escaped.is_whitespace() && escaped != '\0') {
			tokens.push(TokenType::Operator(escaped));
		}
	}

	tokens
}