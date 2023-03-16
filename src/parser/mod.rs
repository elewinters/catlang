use crate::lexer::TokenType::{self, *};

pub type Expression = Vec<TokenType>;

#[derive(Debug)]
pub enum AstType<'a> {
	/* function name, tuple of vectors, first vector holds names, second one holds types, return type */
	FunctionDefinition(&'a str, (Vec<String>, Vec<String>), Option<String>),
	/* function name, vector of types that the function accepts, return type */
	FunctionPrototype(&'a str, Vec<String>, Option<String>),
	/* expression */
	ReturnStatement(Expression),
	/* first expression, operator, second expression */
	IfStatement(Expression, ComparisonOperator, Expression),
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

#[derive(Debug)]
pub enum ComparisonOperator {
	Equal, // == 
	NotEqual, // !=
	GreaterThan, // >
	LessThan, // <
	GreaterThanEqual, // >=
	LessThanEqual // <= 
}

pub fn print_ast(ast: &[AstType]) {
	let mut indent_level = 0;

	fn print_tabs(indent_level: &mut i32) {
		for _ in 0..*indent_level {
			print!("\t");
		}
	}

	for i in ast {
		match (i) {
			AstType::FunctionDefinition(name, (arg_names, arg_types), return_type) => {
				print!("FunctionDefintion(name: {name}, arg_names: {:?}, arg_types: {:?}, return_type: {:?}) {{", arg_names, arg_types, return_type);
				indent_level += 1;
			}

			AstType::IfStatement(_, _, _) => {
				print_tabs(&mut indent_level);
				print!("{:?} {{", i);
				
				indent_level += 1;
			}

			AstType::Newline => println!(),
			AstType::ScopeEnd => {
				indent_level -= 1;

				print_tabs(&mut indent_level);
				print!("}}");
			}
			_ => {
				print_tabs(&mut indent_level);
				print!("{:?} ", i);
			}
		}
	}

	println!();
}

/* all this function does is start from the iterator provided and keep adding every token it sees to a vector until it hits ; */
/* and then it returns that vector */
/* things like return statements and variable declerations use this */
fn seperate_expression(iter: &mut core::slice::Iter<TokenType>, terminator: char) -> Expression {
	let mut expression: Expression = Vec::new();

	for i in iter.by_ref() {
		match i {
			Operator(x) if *x == terminator => break,
			_ => expression.push(i.clone())
		}
	}

	expression
}

/* no idea how this function works i know its extremely messy just dont worry about it */
/* think of it as a little black box that magically processes your function paramaters */
pub fn process_function_parameters(iter: &mut core::slice::Iter<TokenType>) -> Vec<Expression> {
	let mut arguments: Vec<Expression> = Vec::new();
	/* for nested function calls */
	let mut function_levels = 1;

	/* iterate over tokens and push the arguments to 'arguments' vector */
	'outer: while let Some(v) = iter.next() {
		if let Operator(')') = v {
			break;
		}

		let mut expr = Vec::new();
		expr.push(v.clone());

		for v in iter.by_ref() {
			match (v) {
				Operator(',') => {
					if (function_levels <= 1) {
						break;
					}
					
					expr.push(v.clone());
				},
				Operator('(') => {
					function_levels += 1;
					expr.push(v.clone());
				}
				Operator(')') => {
					function_levels -= 1;
					
					if (function_levels == 0) {
						arguments.push(expr);
						break 'outer;
					}

					expr.push(v.clone())
				},
				Newline => (),

				_ => expr.push(v.clone()),

			}
		}
		
		arguments.push(expr);
	}

	arguments
}

pub fn parse(input: &[TokenType]) -> Result<Vec<AstType>, (String, i64)> {
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
			/* ------------------------------------- */
			/*    function definitions/prototypes    */
			/* ------------------------------------- */
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
								Some(Operator(',')) => (),
								Some(Operator(')')) => break,

								Some(Operator('{')) => return Err((format!("unexpected opening curly brace '{{' in paramater list of function definition of {function_name}, did you forget to close the parentheses of the argument list?"), line)),
								_ => return Err((format!("expected a comma after paramater '{varname}'"), line))
							}
						}

						Operator(')') => break,
						err => return Err((format!("expected either an operator ')', operator '{{' or identifier in function definition of '{function_name}', but got {err} instead"), line))
					}
				}

				let mut return_type: Option<String> = None;
				let mut is_proto = false;

				/* determine return type */
				match iter.next() {
					Some(Operator('-')) => {
						/* check for > */
						match iter.next() {
							Some(Operator('>')) => (),
							
							Some(x) => return Err((format!("expected operator '>' after operator '-' in function prototype of '{function_name}', but got {x} instead"), line)),
							None => return Err((format!("expected operator '>' after operator '-' in function prototype of '{function_name}'"), line)),
						}

						/* now get the actual return type */
						return_type = match iter.next() {
							Some(Identifier(x)) => Some(x.to_owned()),

							_ => return Err((format!("expected return type after '->' in function prototype of '{function_name}'"), line))
						};

						match iter.next() {
							Some(Operator('{')) => (),
							Some(Operator(';')) | Some(Newline) => is_proto = true,

							/* unwrap will never panic here */
							_ => return Err((format!("expected '{{' after '-> {}', or a ';'/newline if this is a function prototype", return_type.unwrap()), line))
						}
					}
					Some(Operator('{')) => (),
					Some(Operator(';')) | Some(Newline) => is_proto = true,
					_ => return Err((format!("expected either '{{', '->', ';' or a newline after the paramater list of '{function_name}'"), line))
				}
				
				if (!is_proto) {
					ast.push(AstType::FunctionDefinition(function_name, (arg_names, arg_types), return_type));
				}
				else {
					ast.push(AstType::FunctionPrototype(function_name, arg_types, return_type));
				}
			},
			/* ------------------------ */
			/*    function returning    */
			/* ------------------------ */
			Keyword(keyword) if keyword == "return" => {
				let return_expr = seperate_expression(&mut iter, ';');

				/* push everything to the AST */
				ast.push(AstType::ReturnStatement(return_expr));
			}
			/* ----------------------- */
			/*      if statements      */
			/* ----------------------- */
			Keyword(keyword) if keyword == "if" => {
				match iter.next() {
					Some(Operator('(')) => (),
					_ => return Err((String::from("expected '(' after if keyword"), line))
				};

				let mut expr1: Expression = Vec::new();
				let mut operator: Option<ComparisonOperator> = None;

				while let Some(x) = iter.next() {
					match x {
						/* == */
						Operator('=') => {
							match iter.next() {
								Some(Operator('=')) => {
									operator = Some(ComparisonOperator::Equal);
									break;
								},

								Some(Operator(x)) => return Err((format!("invalid operator '={x}'"), line)),
								Some(x) => return Err((format!("expected '=' after '=', but got {x}"), line)),
								_ => return Err((String::from("expected '=' after '='"), line))
							}
						},
						/* != */
						Operator('!') => {
							match iter.next() {
								Some(Operator('=')) => {
									operator = Some(ComparisonOperator::NotEqual);
									break;
								},

								Some(Operator(x)) => return Err((format!("invalid operator '!{x}'"), line)),
								Some(x) => return Err((format!("expected '=' after '!', but got {x}"), line)),
								_ => return Err((String::from("expected '=' after '!'"), line))
							}
						}
						/* >, >= */
						Operator('>') => {
							if let Some(Operator('=')) = iter.clone().peekable().peek() {
								operator = Some(ComparisonOperator::GreaterThanEqual);
								iter.next();
								break;
							}

							operator = Some(ComparisonOperator::GreaterThan);
							break;
						}
						/* <, <= */
						Operator('<') => {
							if let Some(Operator('=')) = iter.clone().peekable().peek() {
								operator = Some(ComparisonOperator::LessThanEqual);
								iter.next();
								break;
							}
							operator = Some(ComparisonOperator::LessThan);
							break;
						}
						_ => expr1.push(x.clone())
					}
				}

				let operator = match operator {
					Some(x) => x,
					None => return Err((String::from("expected an operator in if statement"), line))	
				};

				let mut expr2 = seperate_expression(&mut iter, '{');

				/* the last element of the expression will be ), which we do not want so we get rid of it */
				match expr2.last() {
					Some(Operator(')')) => expr2.pop(),

					Some(x) => return Err((format!("expected ')' before '{{' in if statement, but got {x}"), line)),
					_ => return Err((String::from("expected ')' before '{{' in if statement"), line))
				};

				ast.push(AstType::IfStatement(expr1, operator, expr2));
			}
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
				let initexpr = seperate_expression(&mut iter, ';');

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name, variable_type, initexpr));
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
				
				let arguments = process_function_parameters(&mut iter);

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