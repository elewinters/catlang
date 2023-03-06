use crate::lexer;
use crate::lexer::TokenType::{self, *};

pub mod expressions;
use expressions::Expression;

#[derive(Debug)]
pub enum AstType<'a> {
	/* function name, tuple of vectors, first vector holds names, second one holds types */
	FunctionDefinition(&'a str, (Vec<String>, Vec<String>)),
	/* function name, vector of types that the function accepts, return type */
	FunctionPrototype(&'a str, Vec<String>, Option<String>),
	/* variable name, type, and initializer value */
	VariableDefinition(&'a str, &'a str, Expression),
	/* macro name, arguments */
	MacroCall(&'a str, Vec<Expression>),
	/* function name, arguments */
	FunctionCall(&'a str, Vec<Expression>),
	/* this is so that functions know when they end */
	ScopeEnd,
	/* for counting the line number in parser.rs */
	Newline
}

pub fn process_function_parmaters(iter: &mut core::slice::Iter<TokenType>, line: i64) -> Result<Vec<Expression>, (String, i64)> {
	let mut arguments: Vec<Expression> = Vec::new();

	/* iterate over tokens and push the arguments to 'arguments' vector */
	while let Some(i) = iter.next() {
		match (i) {						
			StringLiteral(_) | IntLiteral(_) | Identifier(_) => arguments.push(expressions::eval_expression(i, iter, line)?),
			
			Operator(',') | Newline => (),

			Operator(';') => break,
			Operator(')') => break,

			err => return Err((format!("unexpected {} in call to function/macro", lexer::token_to_string(err)), line))
		}
	}

	Ok(arguments)
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
			/* --------------------------- */
			/*     function prototypes     */
			/* --------------------------- */
			Keyword(keyword) if keyword == "extern" => {
				let function_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after extern keyword".to_owned(), line))
				};

				/* check for ( */
				match iter.next() {
					Some(Operator('(')) => (),
					_ => return Err((format!("in function prototype of {function_name}, expected ( after the function name"), line))
				}

				let mut arg_types: Vec<String> = Vec::new();

				while let Some(i) = iter.next() {
					match i {
						Identifier(argtype) => {
							arg_types.push(argtype.to_owned()); 

							match iter.next() {
								Some(Operator(',')) => (),
								Some(Operator(')')) => break,

								Some(x) => return Err((format!("in function prototype of '{function_name}', expected a comma after paramater type '{argtype}', but got {} instead", lexer::token_to_string(x)), line)),
								_ => return Err((format!("in function prototype of '{function_name}', expected a comma after paramater type '{argtype}'"), line))
							}
						}

						Operator(')') => break,
						err => return Err((format!("expected either an operator ')', operator ';', newline or identifier in function prototype of '{function_name}', but got {} instead", lexer::token_to_string(err)), line))
					}
				}

				let mut return_type: Option<String> = None;
				
				/* determine return type */
				if let Some(Operator('-')) = iter.next() {
					/* check for > */
					match iter.next() {
						Some(Operator('>')) => (),
						
						Some(x) => return Err((format!("expected operator '>' after operator '-' in function prototype of '{function_name}', but got {} instead", lexer::token_to_string(x)), line)),
						None => return Err((format!("expected operator '>' after operator '-' in function prototype of '{function_name}'"), line)),
					}

					/* now get the actual return type */
					return_type = match iter.next() {
						Some(Identifier(x)) => Some(x.to_owned()),

						_ => return Err((format!("expected return type after '->' in function prototype of '{function_name}'"), line))
					}
				}
				
				ast.push(AstType::FunctionPrototype(function_name, arg_types, return_type));
			}
			/* ---------------------------- */
			/*     function definitions     */
			/* ---------------------------- */
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

				/* add function arguments if they exist */
				let mut arg_names: Vec<String> = Vec::new();
				let mut arg_types: Vec<String> = Vec::new();

				while let Some(i) = iter.next() {
					match i {
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

							match iter.next() {
								Some(Operator(',')) | Some(Operator(')')) => (),

								Some(Operator('{')) => return Err((format!("unexpected opening curly brace '{{' in paramater list of function definition of {function_name}, did you forget to close the parentheses of the argument list?"), line)),
								_ => return Err((format!("expected a comma after paramater '{varname}'"), line))
							}
						}

						Operator(')') | Operator('{') => break,
						err => return Err((format!("expected either an operator ')', operator '{{' or identifier in function definition of '{function_name}', but got {} instead", lexer::token_to_string(err)), line))
					}
				}
				
				ast.push(AstType::FunctionDefinition(function_name, (arg_names, arg_types)));
			},
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
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
				let intializer_value = expressions::eval_expression(match iter.next() {
					Some(x) => x,
					None => return Err(("expected an initializer value in variable decleration".to_owned(), line))
				}, &mut iter, line)?;

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name, variable_type, intializer_value));
			}
			/* ---------------------------- */
			/*    function/macro calling    */
			/* ---------------------------= */
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
				
				let arguments = process_function_parmaters(&mut iter, line)?;

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