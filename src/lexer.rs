#![allow(unused_parens)]
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Token {
	Keyword(Keyword),
	Identifier(String),

	StringLiteral(String),
	Numerical(String),

	Operator(Operator),
	Newline
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
	Star, Slash, Plus, Dash,

	Equal,

	Colon, Semicolon, Comma,
	Bang,

	LeftParen, RightParen,

	LeftCurly, RightCurly,

	LeftAngle, RightAngle,

	/* multi character operators */
	Arrow, /* -> */

	StarEqual, /* *= */
	SlashEqual, /* /= */
	PlusEqual, /* += */
	DashEqual, /* -= */

	DoubleEqual, /* == */
	BangEqual, /* != */

	LeftAngleEqual, /* <= */
	RightAngleEqual, /* >= */
}

#[derive(Debug, Clone)]
pub enum Keyword {
	Let,
	Fn,
	Return,
	If
}

#[derive(Debug, PartialEq, Clone)]
enum LexerMode {
	Identifier,

	Numerical,
	StringLiteral,

	Operator,

	Newline,
	Comment,
	Ignore
}

impl Operator {
	fn new(input: &str) -> Option<Operator> {
		match input {
			"*" => Some(Operator::Star),
			"/" => Some(Operator::Slash),
			"+" => Some(Operator::Plus),
			"-" => Some(Operator::Dash),
			
			"=" => Some(Operator::Equal),
			":" => Some(Operator::Colon),
			";" => Some(Operator::Semicolon),
			"!" => Some(Operator::Bang),
			"," => Some(Operator::Comma),

			"(" => Some(Operator::LeftParen),
			")" => Some(Operator::RightParen),

			"{" => Some(Operator::LeftCurly),
			"}" => Some(Operator::RightCurly),

			"<" => Some(Operator::LeftAngle),
			">" => Some(Operator::RightAngle),

			_ => None,
		}
	}
}

impl Keyword {
	fn new(input: &str) -> Option<Keyword> {
		match input {
			"let" => Some(Keyword::Let),
			"fn" => Some(Keyword::Fn),
			"return" => Some(Keyword::Return),
			"if" => Some(Keyword::If),

			_ => None
		}
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Token::Keyword(x) => write!(f, "keyword '{:?}'", x),
			Token::Identifier(x) => write!(f, "identifier '{x}'"),

			Token::StringLiteral(x) => write!(f, "string literal '{x}'"),
			Token::Numerical(x) => write!(f, "int literal '{x}'"),

			Token::Operator(x) => write!(f, "operator '{:?}'", x),
			Token::Newline => write!(f, "newline")
		}
	}
}

pub fn print_tokens(input: &Vec<Token>) {
	for i in input {
		match (i) {
			Token::Keyword(x) => print!("Keyword[{:?}], ", x),
			Token::Identifier(x) => print!("Identifier[{}], ", x),
			Token::StringLiteral(x) => print!("StringLiteral[\"{}\"], ", x),
			Token::Numerical(x) => print!("Numerical[{}], ", x),

			Token::Operator(x) => print!("Operator[{:?}], ", x),

			Token::Newline => println!("Newline")
		}
	}
	println!();
}

/* this function joins 2 tokens together into one if it finds a certain pattern */
/* like with for example, multi character operators (arrow operator (->), double equal operator (==)) */
fn join_tokens(tokens: &mut Vec<Token>) {
	/* i am pretty sure this doesnt panic */
	let mut i = 0;
	while (i+1 < tokens.len()) {
		match (&tokens[i], &tokens[i+1]) {
			/* combine - and > into -> */
			(Token::Operator(Operator::Dash), Token::Operator(Operator::RightAngle)) => {
				tokens[i] = Token::Operator(Operator::Arrow);
				tokens.remove(i+1);
			}
			/* combine = and = into == */
			(Token::Operator(Operator::Equal), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::DoubleEqual);
				tokens.remove(i+1);
			}
			/* combine ! and = into != */
			(Token::Operator(Operator::Bang), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::BangEqual);
				tokens.remove(i+1);
			}
			/* combine < and = into <= */
			(Token::Operator(Operator::LeftAngle), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::LeftAngleEqual);
				tokens.remove(i+1);
			}
			/* combine > and = into >= */
			(Token::Operator(Operator::RightAngle), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::RightAngleEqual);
				tokens.remove(i+1);
			}
			/* combine * and = into *= */
			(Token::Operator(Operator::Star), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::StarEqual);
				tokens.remove(i+1);
			}
			/* combine / and = into /= */
			(Token::Operator(Operator::Slash), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::SlashEqual);
				tokens.remove(i+1);
			}
			/* combine + and = into += */
			(Token::Operator(Operator::Plus), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::PlusEqual);
				tokens.remove(i+1);
			}
			/* combine - and = into -= */
			(Token::Operator(Operator::Dash), Token::Operator(Operator::Equal)) => {
				tokens[i] = Token::Operator(Operator::DashEqual);
				tokens.remove(i+1);
			}
			_ => ()
		}
		i += 1;
	}
}

fn push_token(mode: &LexerMode, token: &mut String, tokens: &mut Vec<Token>) -> Result<(), String> {
	match mode {
		LexerMode::Ignore | LexerMode::Comment => (),

		LexerMode::Numerical => tokens.push(Token::Numerical(token.clone())),
		LexerMode::Newline => tokens.push(Token::Newline),

		LexerMode::StringLiteral => {
			token.remove(0); /* remove unnecessary quote at the beginning */
			tokens.push(Token::StringLiteral(token.clone()))
		}

		/* figure out if the operator passed is valid or not */
		LexerMode::Operator => {
			let operator = match Operator::new(token) {
				Some(x) => x,
				None => return Err(format!("'{token}' is not a valid operator"))
			};
			tokens.push(Token::Operator(operator))
		}
		/* this deals with identifiers but also checks for keywords */
		LexerMode::Identifier => {
			/* if its a keyword we push a keyword enum, if its not we just push a regular identifier */
			let to_push = match Keyword::new(token) {
				Some(x) => Token::Keyword(x),
				None => Token::Identifier(token.clone()),
			};

			tokens.push(to_push);
		}
	}

	token.clear();
	Ok(())
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
	let mut iter = input.chars().peekable();

	let mut tokens: Vec<Token> = Vec::new();
	let mut prev_mode = LexerMode::Identifier;

	let mut token = String::new();

	while let Some(i) = iter.next() {
		/* -------------- */
		/*    comments    */
		/* -------------- */
		let new_mode = if (i == '*') {
			if let Some('/') = iter.peek()  {
				iter.next(); /* skip '*' */
				LexerMode::Ignore /* next token (/) gets just ignored */
			}
			else {
				LexerMode::Operator
			}
		}
		else if (prev_mode == LexerMode::Comment) {
			LexerMode::Comment
		}
		else if (i == '/') {
			if let Some('*') = iter.peek() {
				iter.next();
				LexerMode::Comment
			}
			else {
				LexerMode::Operator
			}
		} 
		/* --------------------- */
		/*    string literals    */
		/* --------------------- */
		else if (i == '\"') {
			/* opening quote */
			if (prev_mode != LexerMode::StringLiteral) {
				LexerMode::StringLiteral
			}
			/* closing quote */
			else {
				LexerMode::Ignore /* we wanna ignore the closing quote */
			}
		}
		else if (prev_mode == LexerMode::StringLiteral) {
			LexerMode::StringLiteral
		}
		/* ----------------- */
		/*    identifiers    */
		/* ----------------- */
		else if (i == '!') {
			if (prev_mode == LexerMode::Identifier) {
				LexerMode::Identifier
			}
			else {
				LexerMode::Operator
			}
		}
		else if (i.is_alphabetic() || i == '_') {
			LexerMode::Identifier
		}
		/* ---------------- */
		/*    numericals    */
		/* ---------------- */
		else if (i.is_alphanumeric()) {
			/* this is for handling identifiers that have numbers in them, like i32 or meow20 */
			/* this checks if the state we handled last time is an identifier, if yes, we keep it that way */
			/* if this if expression didnt exist here and we tried to lex "let x: i32;" we would get [Keyword(let), Identifier(x), Identifier(i), Numerical(32)] */
			if (prev_mode == LexerMode::Identifier) {
				LexerMode::Identifier
			}
			else {
				LexerMode::Numerical
			}
		}
		/* --------------- */
		/*    operators    */
		/* --------------- */
		else if (i.is_ascii_punctuation()) {
			LexerMode::Operator
		}
		/* ---------------- */
		/*    whitespace    */
		/* ---------------- */
		else if (i == '\n') {
			LexerMode::Newline
		}
		else if (i.is_whitespace()) {
			LexerMode::Ignore
		}
		else {
			prev_mode.clone()
		};

		/* if we are now in a different state, push what everything we pushed into 'token' into the 'tokens' vector */
		/* unless the state is Operator, we want to update tokens on every iteration if we're in the Operator state */
		if (prev_mode != new_mode || prev_mode == LexerMode::Operator) {			
			push_token(&prev_mode, &mut token, &mut tokens)?;			
			prev_mode = new_mode;
		}

		token.push(i);
	}

	/* we still need to add one more token that the iteration didnt go through*/
	push_token(&prev_mode, &mut token, &mut tokens)?;
	join_tokens(&mut tokens);

	Ok(tokens)
}