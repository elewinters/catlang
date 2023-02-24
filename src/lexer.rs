#[derive(Debug)]
pub enum TokenType {
	Keyword(String),
	Identifier(String),

	StringLiteral(String),
	IntLiteral(String),

	Operator(char),
}

impl TokenType {
	pub fn human_readable(&self) -> String {
		match &self {
			TokenType::Keyword(x) => format!("keyword '{x}'"),
			TokenType::Identifier(x) => format!("identifier '{x}'"),

			TokenType::StringLiteral(x) => format!("string literal '{x}'"),
			TokenType::IntLiteral(x) => format!("int literal '{x}'"),

			TokenType::Operator(x) => format!("operator '{x}'"),
		}
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

			TokenType::Operator(x) => {
				print!("Operator[{}], ", x);
				
				match (*x) {
					';' | '{' | '}' => println!(),
					_ => ()
				}
			}
		}
	}
}

pub fn lex(input: Vec<u8>) -> Vec<TokenType> {
	let mut tokens: Vec<TokenType> = Vec::new();
	let mut iter = input.iter();
	
	while let Some(v) = iter.next() {
		let v = *v as char;
		
		if (v.is_whitespace()) {
			continue;
		}

		if (v.is_ascii_punctuation() && v != '"') {
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
		else if (v.is_ascii_alphabetic()) {
			tokens.push(TokenType::Identifier(identifier));
		}
		else if (v.is_alphanumeric()) {
			tokens.push(TokenType::IntLiteral(identifier));
		}
		else if (v == '"') {
			identifier.remove(0);
			tokens.push(TokenType::StringLiteral(identifier));
		}
		
		if (!escaped.is_whitespace() && escaped != '\0') {
			tokens.push(TokenType::Operator(escaped));
		}
	}

	tokens
}