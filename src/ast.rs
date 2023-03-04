use crate::lexer::{TokenType, token_to_string};
use crate::lexer::TokenType::*;
use crate::expressions;

#[derive(Debug)]
pub enum AstType<'a> {
	/* function name, tuple of vectors, first vector holds names, second one holds types */
	FunctionDefinition(&'a str, (Vec<String>, Vec<String>)),
	/* function name, vector of types that the function accepts */
	FunctionPrototype(&'a str, Vec<String>),
	/* variable name, type, and initializer value */
	VariableDefinition(&'a str, &'a str, Expression),
	/* macro name, arguments */
	MacroCall(&'a str, Vec<&'a TokenType>),
	/* function name, arguments */
	FunctionCall(&'a str, Vec<&'a TokenType>),
	/* this is so that functions know when they end */
	ScopeEnd,
	/* for counting the line number in parser.rs */
	Newline
}

#[derive(Debug)]
pub enum Expression {
	NumericalExpression(String),
	StringExpression(String),
	FunctionCall(String, Vec<Expression>),
}

pub fn ast(input: &[TokenType]) -> Result<Vec<AstType>, (String, i64)> {
	let mut ast: Vec<AstType> = Vec::new();
	let mut line: i64 = 1;

	let mut iter = input.iter();
	while let Some(i) = iter.next() {
		match i {
			/* increment line number on newline token */
			Newline => {
				line += 1;
				ast.push(AstType::Newline);
			},
			/* scope end */
			Operator('}') => ast.push(AstType::ScopeEnd),
			/* function definitions */
			Keyword(keyword) if keyword == "fn" => {
				let function_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after function keyword".to_owned(), line))
				};

				/* check for ( */
				match iter.next() {
					Some(Operator('(')) => (),
					_ => return Err((format!("in function definition of {function_name}, expected ( after the function name"), line))
				}

				let mut arg_names: Vec<String> = Vec::new();
				let mut arg_types: Vec<String> = Vec::new();

				let mut is_proto: bool = false;

				/* add function arguments if they exist */
				while let Some(i) = iter.next() {
					match i {
						/* push arg_name: arg_type to vectors */
						Identifier(varname) => {
							match iter.next() {
								Some(Operator(':')) => (),
								_ => return Err((format!("expected an operator ':' after function paramater '{varname}'"), line)) 
							}

							arg_names.push(varname.to_owned()); 
							arg_types.push(match iter.next() {
								Some(Identifier(vartype)) => vartype.to_owned(),
								_ => return Err((format!("expected a type after paramater name '{varname}' in function decleration of {function_name}"), line))
							});

							/* syntax error checks */
							match iter.next() {
								Some(Operator(',')) | Some(Operator(')')) => (),

								Some(Operator('{')) => return Err((format!("unexpected opening curly brace '{{' in paramater list of function definition of {function_name}, did you forget to close the parentheses of the argument list?"), line)),
								_ => return Err((format!("expected an operator ',', operator ')' or operator '{{' after paramater '{varname}'"), line))
							}
						}
						
						/* determine when to stop */
						/* stuff for determining whether it's a prototype only happens here */
						Operator('{') => {
							is_proto = false; 
							break;
						},
						Operator(')') => {
							match iter.next() {
								Some(Operator('{')) => is_proto = false,
								Some(Operator(';')) | None => is_proto = true,

								Some(x) => return Err((format!("expected either an operator '{{' or an operator ';' after operator ')', but got {} instead", token_to_string(x)), line))
							}

							break;
						}
						Operator(';') | Newline => {
							is_proto = true; 
							break;
						}

						err => return Err((format!("expected either an operator ')', operator '{{', operator ';', operator 'newline' or identifier in function definition of '{function_name}', but got {} instead", token_to_string(err)), line))
					}
				}
				
				if (!is_proto) {
					ast.push(AstType::FunctionDefinition(function_name, (arg_names, arg_types)));
				}
				else {
					ast.push(AstType::FunctionPrototype(function_name, arg_types));
				}
			},
			/* variable declerations */
			Keyword(keyword) if keyword == "let" => {
				/* get variable name */
				let variable_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after let keyword".to_owned(), line))
				};

				/* check if there's a colon after variable name */
				match iter.next() {
					Some(Operator(':')) => (),
					_ => return Err(("expected operator ':' token after the variable name".to_owned(), line))
				};
				
				/* get variable type */
				let variable_type = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after operator ':'".to_owned(), line))
				};

				/* check if there's a = after the type name */
				match iter.next() {
					Some(Operator('=')) => (),
					_ => return Err(("expected operator '=' after type name".to_owned(), line))
				};

				/* get initializer value */
				let intializer_value = expressions::eval_expression(&mut iter, line)?;

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name, variable_type, intializer_value));
			}
			/* macro/function calls */
			Identifier(identifier) => {
				let macro_or_function = if identifier.ends_with('!') {
					"macro"
				}
				else {
					"function"
				};

				/* check for ( */
				match iter.next() {
					Some(Operator('(')) => (),
					_ => return Err((format!("expected operator '(' after {macro_or_function} '{identifier}'"), line))
				}

				let mut arguments: Vec<&TokenType> = Vec::new();

				/* iterate over tokens and push the arguments to 'arguments' vector */
				while let Some(i) = iter.next() {
					let token = match i {
						Operator(')') => break,
						Operator(';') => return Err((format!("expected a closing ) in call to {macro_or_function} {identifier}"), line)),
						x => x
					};

					match (token) {
						StringLiteral(_) | IntLiteral(_) | Identifier(_) => arguments.push(token),
						Operator(',') | Newline => (),
						err => return Err((format!("unexpected {} in call to {macro_or_function} {identifier}", token_to_string(err)), line))
					}
				}

				if (macro_or_function == "macro") {
					ast.push(AstType::MacroCall(identifier, arguments));
				}
				else {
					ast.push(AstType::FunctionCall(identifier, arguments));
				}
			}
			_ => (),
		}
	}

	Ok(ast)
}