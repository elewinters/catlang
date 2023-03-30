use crate::lexer::Token::{self, *};
use crate::lexer::{Operator::{self, *}, Keyword};
use crate::exit;

pub type Expression = Vec<Token>;
pub type BlockStatement = Vec<AstType>;

#[derive(Debug)]
pub enum AstType {
	/* function name, tuple of vectors, first vector holds names, second one holds types, return type, body */
	FunctionDefinition(String, (Vec<String>, Vec<String>), Option<String>, BlockStatement),
	/* function name, vector of types that the function accepts, return type */
	FunctionPrototype(String, Vec<String>, Option<String>),
	/* expression */
	ReturnStatement(Expression),
	/* first expression, operator, second expression, body */
	IfStatement(Expression, ComparisonOperator, Expression, BlockStatement),
	/* variable name, type, and initializer value */
	VariableDefinition(String, Option<String>, Option<Expression>),
	/* assigning a value to an already existing variable, like 'num = 5' */
	/* variable name, assignment expression */
	VariableAssigment(String, Expression),
	/* macro name, arguments */
	MacroCall(String, Vec<Expression>),
	/* function name, arguments */
	FunctionCall(String, Vec<Expression>),
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

pub fn print_ast(ast: &[AstType], indent_levels: u64) {
	for i in ast {
		for _ in 0..indent_levels {
			print!("\t");
		}

		match (i) {
			AstType::FunctionDefinition(name, (arg_names, arg_types), return_type, body) => {
				print!("FunctionDefintion(name: {name}, arg_names: {:?}, arg_types: {:?}, return_type: {:?}) {{", arg_names, arg_types, return_type);
				print_ast(body, 1);

				print!("}}");			}

			AstType::IfStatement(_, _, _, body) => {
				print!("{:?} {{", i);
				print_ast(body, 2);

				print!("\t}}");
			}

			AstType::Newline => println!(),
			_ => {
				print!("{:?} ", i);
			}
		}
	}
}

/* all this function does is start from the iterator provided and keep adding every token it sees to a vector until it hits ; */
/* and then it returns that vector */
/* things like return statements and variable declerations use this */
fn seperate_expression(iter: &mut core::slice::Iter<Token>, terminator: &Operator) -> Expression {
	let mut expression: Expression = Vec::new();

	for i in iter.by_ref() {
		match i {
			/* if we stumble upon a newline and the input was Semicolon, that means we should stop */
			Newline if terminator == &Semicolon => break,
			Operator(x) if x == terminator => break,
			
			_ => expression.push(i.clone())
		}
	}

	expression
}

fn seperate_block_statement(iter: &mut core::slice::Iter<Token>, line: i64) -> BlockStatement {
	let mut block_statement_tokens = Vec::new();
	let mut scopes: u64 = 1; /* if this is initialized to 0 we will overflow */

	for i in iter.by_ref() {
		match i {
			Operator(LeftCurly) => {
				block_statement_tokens.push(i.clone());
				scopes += 1
			},
			Operator(RightCurly) => {
				scopes -= 1;
				if (scopes == 0) {
					break;
				}
				else {
					block_statement_tokens.push(i.clone());
				}
			},
			_ => block_statement_tokens.push(i.clone())
		}
	}

	/* we return here if we parsed the tokens successfully */
	match parse(block_statement_tokens) {
		Ok(x) => x,
		Err((err, errline)) => exit!(format!("[line {}] {}", (line+1)+(errline+1)+2, err))
	}
}

/* no idea how this function works i know its extremely messy just dont worry about it */
/* think of it as a little black box that magically processes your function paramaters */
pub fn process_function_parameters(iter: &mut core::slice::Iter<Token>) -> Vec<Expression> {
	let mut arguments: Vec<Expression> = Vec::new();
	/* for nested function calls */
	let mut function_levels = 1;

	/* iterate over tokens and push the arguments to 'arguments' vector */
	'outer: while let Some(v) = iter.next() {
		if let Operator(RightParen) = v {
			break;
		}

		let mut expr = Vec::new();
		expr.push(v.clone());

		for v in iter.by_ref() {
			match (v) {
				Operator(Comma) => {
					if (function_levels <= 1) {
						break;
					}
					
					expr.push(v.clone());
				},
				Operator(LeftParen) => {
					function_levels += 1;
					expr.push(v.clone());
				}
				Operator(RightParen) => {
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

pub fn parse(input: Vec<Token>) -> Result<Vec<AstType>, (String, i64)> {
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
			/* ------------------------------------- */
			/*    function definitions/prototypes    */
			/* ------------------------------------- */
			Keyword(Keyword::Fn) => {
				let function_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after function keyword".to_owned(), line))
				};

				/* check for ( */
				match iter.next() {
					Some(Operator(LeftParen)) => (),
					_ => return Err((format!("in function definition of {function_name}, expected ( after the function name"), line))
				}

				/* add function arguments if they exist */
				let mut arg_names: Vec<String> = Vec::new();
				let mut arg_types: Vec<String> = Vec::new();

				while let Some(i) = iter.next() {
					match i {
						Identifier(varname) => {
							match iter.next() {
								Some(Operator(Colon)) => (),
								_ => return Err((format!("expected an operator ',' after function paramater '{varname}'"), line)) 
							}

							arg_names.push(varname.to_owned()); 
							arg_types.push(match iter.next() {
								Some(Identifier(vartype)) => vartype.to_owned(),
								_ => return Err((format!("expected a type after paramater name '{varname}' in function decleration of {function_name}"), line))
							});

							match iter.next() {
								Some(Operator(Comma)) => (),
								Some(Operator(RightParen)) => break,

								Some(Operator(LeftCurly)) => return Err((format!("unexpected opening curly brace '{{' in paramater list of function definition of {function_name}, did you forget to close the parentheses of the argument list?"), line)),
								_ => return Err((format!("expected a comma after paramater '{varname}'"), line))
							}
						}

						Operator(RightParen) => break,
						err => return Err((format!("expected either an operator RightParen, operator '{{' or identifier in function definition of '{function_name}', but got {err} instead"), line))
					}
				}

				let mut return_type: Option<String> = None;
				let mut is_proto = false;

				/* determine return type */
				match iter.next() {
					Some(Operator(Arrow)) => {
						/* get the actual return type */
						return_type = match iter.next() {
							Some(Identifier(x)) => Some(x.to_owned()),

							_ => return Err((format!("expected return type after '->' in function prototype of '{function_name}'"), line))
						};

						match iter.next() {
							Some(Operator(LeftCurly)) => (),
							Some(Operator(Semicolon)) | Some(Newline) => is_proto = true,

							/* unwrap will never panic here */
							_ => return Err((format!("expected '{{' after '-> {}', or a Semicolon/newline if this is a function prototype", return_type.unwrap()), line))
						}
					}
					Some(Operator(LeftCurly)) => (),
					Some(Operator(Semicolon)) | Some(Newline) => is_proto = true,
					_ => return Err((format!("expected either '{{', '->', Semicolon or a newline after the paramater list of '{function_name}'"), line))
				}

				if (is_proto) {
					ast.push(AstType::FunctionPrototype(function_name.to_owned(), arg_types, return_type));
					continue;
				}
				
				let block_statement = seperate_block_statement(&mut iter, line);
				println!("block statement of {}: {:?}", function_name, block_statement);
				
				ast.push(AstType::FunctionDefinition(function_name.to_owned(), (arg_names, arg_types), return_type, block_statement));
			},
			/* ------------------------ */
			/*    function returning    */
			/* ------------------------ */
			Keyword(Keyword::Return) => {
				let return_expr = seperate_expression(&mut iter, &Semicolon);

				/* push everything to the AST */
				ast.push(AstType::ReturnStatement(return_expr));
			}
			/* ----------------------- */
			/*      if statements      */
			/* ----------------------- */
			Keyword(Keyword::If) => {
				match iter.next() {
					Some(Operator(LeftParen)) => (),
					_ => return Err((String::from("expected LeftParen after if keyword"), line))
				};

				let mut expr1: Expression = Vec::new();
				let mut operator: Option<ComparisonOperator> = None;

				for i in iter.by_ref() {
					match i {
						/* == */
						Operator(DoubleEqual) => {
							operator = Some(ComparisonOperator::Equal);
							break;
						},
						/* != */
						Operator(BangEqual) => {
							operator = Some(ComparisonOperator::NotEqual);
							break;
						}
						/* > */
						Operator(RightAngle) => {
							operator = Some(ComparisonOperator::GreaterThan);
							break;
						}
						/* >= */
						Operator(RightAngleEqual) => {
							operator = Some(ComparisonOperator::GreaterThanEqual);
							break;
						}
						/* < */
						Operator(LeftAngle) => {
							operator = Some(ComparisonOperator::LessThan);
							break;
						}
						/* <= */
						Operator(LeftAngleEqual) => {
							operator = Some(ComparisonOperator::LessThanEqual);
							break;
						}
						_ => expr1.push(i.clone())
					}
				}

				let operator = match operator {
					Some(x) => x,
					None => return Err((String::from("expected an operator in if statement"), line))	
				};

				let mut expr2 = seperate_expression(&mut iter, &LeftCurly);
				let block_statement = seperate_block_statement(&mut iter, line);

				/* the last element of the expression will be ), which we do not want so we get rid of it */
				match expr2.last() {
					Some(Operator(RightParen)) => expr2.pop(),

					Some(x) => return Err((format!("expected RightParen before '{{' in if statement, but got {x}"), line)),
					_ => return Err((String::from("expected RightParen before '{{' in if statement"), line))
				};

				ast.push(AstType::IfStatement(expr1, operator, expr2, block_statement));
			}
			/* --------------------------- */
			/*    variable declerations    */
			/* --------------------------- */
			Keyword(Keyword::Let) => {
				/* get variable name */
				let variable_name = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after let keyword".to_owned(), line))
				};

				/* check if there's a colon after variable name */
				match iter.next() {
					Some(Operator(Colon)) => (),
					/* if theres a = instead of a : that means we should type infer this */
					Some(Operator(Equal)) => {
						let initexpr = seperate_expression(&mut iter, &Semicolon);
						ast.push(AstType::VariableDefinition(variable_name.to_owned(), None, Some(initexpr)));

						continue;
					}
					_ => return Err(("expected operator ',' or operator Equal after the variable name".to_owned(), line))
				};
				
				/* get variable type */
				let variable_type = match iter.next() {
					Some(Identifier(x)) => x,
					_ => return Err(("expected identifier after operator ',".to_owned(), line))
				};

				/* check if there's a = after the type name */
				match iter.next() {
					Some(Operator(Equal)) => (),
					Some(Operator(Semicolon) | Newline) => {
						ast.push(AstType::VariableDefinition(variable_name.to_owned(), Some(variable_type.to_owned()), None));
						continue;
					}
					_ => return Err(("expected operator '=', operator ';' or newline after type name".to_owned(), line))
				};

				/* get initializer value */
				let initexpr = seperate_expression(&mut iter, &Semicolon);

				/* push everything to the AST */
				ast.push(AstType::VariableDefinition(variable_name.to_owned(), Some(variable_type.to_owned()), Some(initexpr)));
			}
			/* -------------------------------------------------- */
			/*    function/macro calling + variable assignment    */
			/* -------------------------------------------------- */
			Identifier(identifier) => {
				match iter.next() {
					Some(Operator(LeftParen)) => {
						let arguments = process_function_parameters(&mut iter);

						if (identifier.ends_with('!')) {
							ast.push(AstType::MacroCall(identifier.to_owned(), arguments));
						}
						else {
							ast.push(AstType::FunctionCall(identifier.to_owned(), arguments));
						}
					},
					Some(Operator(Equal)) => {
						let expr = seperate_expression(&mut iter, &Semicolon);

						ast.push(AstType::VariableAssigment(identifier.to_owned(), expr));
					},
					/* arithmetic assignment operators, like +=, -=, *= and /= */
					Some(Operator(x)) => {
						let op = match x {
							StarEqual => Star,
							SlashEqual => Slash,
							PlusEqual => Plus,
							DashEqual => Dash,
							err => return Err((format!("expected either *=, /=, += or -= after identifier, but got '{:?}'", err), line))
						};

						let mut expr = seperate_expression(&mut iter, &Semicolon);
						/* insert '{identifier} {operator} to the start of the expression ' */
						/* like 'num + ' */
						expr.insert(0, Operator(op));
						expr.insert(0, Identifier(identifier.clone()));

						ast.push(AstType::VariableAssigment(identifier.to_owned(), expr));
					}
					Some(x) => return Err((format!("expected either operator LeftParen or operator Equal after identifier {identifier} but got '{x}'"), line)),
					None => return Err((format!("expected either operator LeftParen or operator Equal after identifier {identifier}, but got nothing"), line))
				}
			}
			Operator(Semicolon) => (),
			err => return Err((format!("unexpected stray {err}"), line)),
		}
	}

	Ok(ast)
}