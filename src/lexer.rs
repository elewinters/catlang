#[derive(Debug)]
pub enum TokenType {
	Keyword(String),
	Identifier(String),

	StringLiteral(String),
	IntLiteral(String),

	Operator(char),
	Newline
}

pub fn token_to_string(token: &TokenType) -> String {
	match token {
		TokenType::Keyword(x) => format!("keyword '{x}'"),
		TokenType::Identifier(x) => format!("identifier '{x}'"),

		TokenType::StringLiteral(x) => format!("string literal '{x}'"),
		TokenType::IntLiteral(x) => format!("int literal '{x}'"),

		TokenType::Operator(x) => format!("operator '{x}'"),
		TokenType::Newline => String::from("'newline'")
	}
}

pub fn token_get_value(token: &TokenType) -> String {
	match token {
		TokenType::Keyword(x) => x.to_owned(),
		TokenType::Identifier(x) => x.to_owned(),

		TokenType::StringLiteral(x) => x.to_owned(),
		TokenType::IntLiteral(x) => x.to_owned(),

		TokenType::Operator(x) => String::from(*x),
		TokenType::Newline => String::from("\n")
	}
}

const KEYWORDS: [&str; 3] = [
	"let", 
	"extern", 
	"fn"
];

const ESCAPE_CHARACTERS: [char; 8] = [
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
}

pub fn lex(input: Vec<u8>) -> Vec<TokenType> {
	let mut tokens: Vec<TokenType> = Vec::new();
	let mut iter = input.iter();
	
	while let Some(v) = iter.next() {
		let v = *v as char;

		if (v == '\n') {
			tokens.push(TokenType::Newline);
		}
		
		if (v.is_whitespace()) {
			continue;
		}

		if (v.is_ascii_punctuation() && v != '"' && v != '_') {
			tokens.push(TokenType::Operator(v));
		}
		
		let mut identifier = String::new();
		/* iter.next() below immediately skips, so we have */
		/* to add this before that happens */
		identifier.push(v);
		
		let mut escaped: char = '\0';
		let mut is_string: bool = false;
		for (i, c) in iter.by_ref().enumerate() {
			if (v == '"') {
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
		else if (v.is_ascii_alphabetic() || v == '_') {
			tokens.push(TokenType::Identifier(identifier));
		}
		else if (v.is_alphanumeric()) {
			tokens.push(TokenType::IntLiteral(identifier));
		}
		else if (v == '"') {
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