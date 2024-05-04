use std::clone::Clone;
use std::collections::VecDeque;
use std::fmt::Display;
use std::io;
use std::iter::Iterator;

macro_rules! map {
	() => {
		HashMap::new()
	};
	($($x:expr),+ $(,)?) => {
		[
			$($x),+
		].iter().collect();
	};
}

#[derive(Copy, Clone, Debug)]
pub struct Operator {
	pub symbol: char,
	pub argc: usize,
	pub precedence: usize,
	resolver: fn(args: &Vec<f64>) -> f64
}
impl Operator {
	pub const MAP: [(char, Self); 4] = [
		('/', Self { symbol: '/', argc: 2, precedence: 4, resolver: |args| {
			args.get(1).unwrap() / args.get(0).unwrap()
		}}),
		('*', Self { symbol: '*', argc: 2, precedence: 3, resolver: |args| {
			args.get(1).unwrap() * args.get(0).unwrap()
		} }),
		('+', Self { symbol: '+', argc: 2, precedence: 2, resolver: |args| {
			args.get(1).unwrap() + args.get(0).unwrap()
		} }),
		('-', Self { symbol: '-', argc: 2, precedence: 1, resolver: |args| {
			args.get(1).unwrap() - args.get(0).unwrap()
		} })
	];

	pub fn by_char(c: char) -> Option<Self> {
		if let Ok(result) = 
			Self::MAP.binary_search_by(|(k, _)| k.cmp(&c)).map(|x| Self::MAP[x].1) {
			return Some(result);
		}
		return None;
	}
	pub fn resolve(&self, args: &Vec<f64>) -> f64 {
		(self.resolver)(args)
	}
}

#[derive(Copy, Clone, Debug)]
enum Token {
	NumericLiteral(f64),
	Operator(Operator),
	OpenParen
}

#[derive(Copy, Clone, Debug)]
enum EvalError {
	InvalidCharacter,
	UnexpectedToken(Token),
	DuplicateDecimal,
	NumberParseError,
	MismatchedParenthesis,
	NotEnoughArguments,
	NoResult
}

fn eval(expression: &str) -> Result<f64, EvalError> {
	let mut holding = VecDeque::new();
	let mut output = VecDeque::new();
	let mut temp = String::new();
	let mut last_token = None;

	for c in expression.chars() {
		if c.is_digit(10) {
			temp.push(c);
		} else if c == '.' {
			if temp.contains('.') {
				return Err(EvalError::DuplicateDecimal);
			}
			temp.push(c);
		} else {
			if !temp.is_empty() {
				let val = temp.parse::<f64>();
				if val.is_err() {
					return Err(EvalError::NumberParseError);
				}
				output.push_back(Token::NumericLiteral(val.unwrap()));
				last_token = output.back().cloned();
				temp.clear();
			}
			if c == '(' {
				holding.push_front(Token::OpenParen);
				last_token = holding.front().cloned();
			} else if c == ')' {
				while !holding.is_empty() {
					if let Some(Token::OpenParen) = holding.front() {
						break;
					}
					output.push_back(holding.pop_front().unwrap());
				}
				if holding.is_empty() {
					return Err(EvalError::MismatchedParenthesis);
				}
				last_token = holding.front().cloned();
				if let Some(Token::OpenParen) = holding.front().cloned() {
					holding.pop_front();
				}
			} else if !c.is_whitespace() {
				if let Some(mut op) = Operator::by_char(c) {
					if op.symbol == '+' || op.symbol == '-' {
						match last_token {
							Some(Token::Operator(_)) | None => {
								op.argc = 1;
								op.precedence = 255;
							}
							_ => { /* Do nothing */ }
						}
					}
					while !holding.is_empty() {
						if let Some(Token::OpenParen) = holding.front() {
							break;
						}
						if let Some(Token::Operator(op_prev)) = holding.front() {
							if op_prev.precedence >= op.precedence {
								output.push_back(holding.pop_front().unwrap());
							} else {
								break;
							}
						}
					}
					holding.push_front(Token::Operator(op));
					last_token = holding.front().cloned();
				} else {
					return Err(EvalError::InvalidCharacter);
				}
			}
		}
	}

	if !temp.is_empty() {
		let val = temp.parse::<f64>();
		if val.is_err() {
			return Err(EvalError::NumberParseError);
		}
		output.push_back(Token::NumericLiteral(val.unwrap()));
		temp.clear();
	}

	while !holding.is_empty() {
		if let Some(tok) = holding.pop_front() {
			output.push_back(tok);
		}
	}

	for tok in &output {
		match tok {
			Token::NumericLiteral(num) => {
				print!("{} ", *num)
			}
			Token::Operator(op) => {
				print!("{} ", op.symbol);
			}
			Token::OpenParen => {
				print!("( ");
			}
		}
	}
	println!();

	let mut solve = VecDeque::new();
	for tok in output {
		match tok {
			Token::NumericLiteral(num) => {
				solve.push_front(num);
			},
			Token::Operator(op) => {
				if solve.len() < op.argc {
					return Err(EvalError::NotEnoughArguments);
				}

				if op.symbol == '+' && op.argc == 1 {
					/* Nothing to do */
				} else if op.symbol == '-' && op.argc == 1 {
					let value = solve.pop_front().unwrap();
					solve.push_front(-value);
				} else {
					let mut args = Vec::with_capacity(op.argc);
					for _ in 0 .. op.argc {
						args.push(solve.pop_front().unwrap());
					}
					if args.len() < op.argc {
						return Err(EvalError::NotEnoughArguments);
					}
					solve.push_front(op.resolve(&args));
					args.clear();
				}
			},
			Token::OpenParen => {
				return Err(EvalError::UnexpectedToken(tok));
			}
		}
	}

	if !solve.is_empty() {
		Ok(*solve.front().unwrap())
	} else {
		Err(EvalError::NoResult)
	}
}

fn main() {
	loop {
		let stdin = io::stdin();
		let mut expr = String::new();
		if stdin.read_line(&mut expr).is_ok() {
			if expr == "end" {
				break;
			}
			match eval(expr.as_str()) {
				Ok(result) => { println!("{} = {}", expr.trim_end(), result); }
				Err(err) => { println!("Error: {:?}", err); }
			}
		} else {
			println!("Input error!");
		}
	}
}
