// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{collections::HashMap, fmt::Debug};

#[derive(Clone)]
pub struct SourceSpan<'a> {
	line: usize,
	snip: &'a str,
}

#[derive(Debug, thiserror::Error)]
#[error("{span:?}: {ty}")]
pub struct PreprocError<'a> {
	ty: PreprocErrorType,
	span: SourceSpan<'a>,
}

#[derive(Debug, thiserror::Error)]
pub enum PreprocErrorType {
	#[error("unknown directive")]
	UnknownDirective,
	#[error("malformed directive: {0}")]
	Malformed(&'static str),
	#[error("match target undefined: {0}")]
	UndefinedTarget(String),
	#[error("case not covered: {0}")]
	MissedCase(String),
	#[error("duplicate case: {0}")]
	DuplicateCase(String),
	#[error("undefined substitution: ${0}")]
	Undefined(String),
	#[error("{0}")]
	Other(&'static str),
}

impl Debug for SourceSpan<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: {}", self.line + 1, self.snip)
	}
}

pub fn preprocess(
	source: &str,
	mut defines: HashMap<String, String>,
) -> Result<String, PreprocError> {
	struct MatchDirective {
		target_case: String,
		hit_cases: Vec<String>,
	}

	enum WriteState {
		Error(&'static str),
		Skip,
		Write,
	}

	enum PreprocToken<'a, 'd> {
		Match(MatchDirective),
		_TODO(std::marker::PhantomData<(&'a (), &'d ())>),
	}

	struct PreprocEntry<'a, 'd> {
		span: SourceSpan<'a>,
		token: PreprocToken<'a, 'd>,
		write: WriteState,
	}

	let mut source_buffer = String::with_capacity(source.len());
	let mut token_stack = Vec::<PreprocEntry>::new();

	for (y, line) in source.lines().enumerate() {
		if let Some('@') = line.trim_start().chars().next() {
			let span = SourceSpan {
				line: y,
				snip: line,
			};

			let directive = &line.trim_start()[1..].trim_start();
			let (command, args) = directive
				.split_once(" ")
				.map(|(c, r)| (c, Some(r)))
				.unwrap_or_else(|| (directive, None));

			match command {
				"match" => {
					let condition = match args {
						Some(x) => x.trim(),
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing match target"),
								span,
							}),
					};

					// get the match's target case from the defines list
					let target_case = match defines.get(condition) {
						Some(x) => x,
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::UndefinedTarget(condition.to_owned()),
								span,
							}),
					};

					token_stack.push(PreprocEntry {
						token: PreprocToken::Match(MatchDirective {
							target_case: target_case.clone(),
							hit_cases: Vec::new(),
						}),
						write: WriteState::Error(
							"code in a match block is only allowed inside case blocks",
						),
						span,
					});
				},
				"case" => {
					let (match_directive, write_block) = match token_stack.last_mut() {
						Some(PreprocEntry {
							token: PreprocToken::Match(match_directive),
							write,
							..
						}) => (match_directive, write),
						Some(_) | None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Other(
									"case directive can only exist inside match",
								),
								span,
							}),
					};

					// split out cases in `a | b | c` form
					let cases = match args {
						Some(x) => x.split('|').into_iter().map(|c| c.trim()).collect::<Vec<_>>(),
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing case list"),
								span,
							}),
					};

					// error if there are any duplicate cases, otherwise add them to hit list
					for case in &cases {
						if match_directive.hit_cases.iter().any(|h| h == case) {
							return Err(PreprocError {
								ty: PreprocErrorType::DuplicateCase(format!("{}", case)),
								span,
							})
						} else {
							match_directive.hit_cases.push(case.to_string());
						}
					}

					// let following lines go into
					*write_block = match cases.iter().any(|c| c == &match_directive.target_case) {
						true => WriteState::Write,
						false => WriteState::Skip,
					};
				},
				"endmatch" => {
					let (match_directive, match_span) = match token_stack.last() {
						Some(PreprocEntry {
							token: PreprocToken::Match(match_directive),
							span,
							..
						}) => (match_directive, span),
						Some(_) | None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Other(
									"endmatch directive can only exist after match",
								),
								span,
							}),
					};

					// error if we missed a case used in a preproc macro
					if !match_directive.hit_cases.iter().any(|h| h == &match_directive.target_case)
					{
						return Err(PreprocError {
							ty: PreprocErrorType::MissedCase(
								match_directive.target_case.to_owned(),
							),
							span: match_span.clone(),
						})
					}

					token_stack.pop();
				},
				"define" => {
					let (key, val) = match args {
						Some(x) => match x.trim().split_once(" ") {
							Some((key, val)) => (key, val),
							None =>
								return Err(PreprocError {
									ty: PreprocErrorType::Malformed("missing definition value"),
									span,
								}),
						},
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing definition"),
								span,
							}),
					};

					// allow defines to work with branches, WriteState::Error is ignored.
					let should_define = match token_stack.last() {
						Some(x) => match x.write {
							WriteState::Skip => false,
							_ => true,
						},
						None => true,
					};

					if should_define {
						defines.insert(key.to_owned(), val.to_owned());
					}
				},
				_ =>
					return Err(PreprocError {
						ty: PreprocErrorType::UnknownDirective,
						span,
					}),
			}
		} else {
			let mut write_str = || {
				let mut sub = line;
				let mut pi = sub.find('$');
				while let Some(i) = pi {
					source_buffer.push_str(&sub[..i]);
					sub = &sub[i + 1..];

					let (define, rest) = match sub.find(' ') {
						Some(i) => (&sub[..i], &sub[i..]),
						None => (sub, ""),
					};
					sub = rest;

					let value = match defines.get(define) {
						Some(x) => x,
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Undefined(define.to_owned()),
								span: SourceSpan {
									line: y,
									snip: line,
								},
							}),
					};

					source_buffer.push_str(value);
					pi = sub.find('$');
				}

				source_buffer.push_str(sub);
				source_buffer.push('\n');
				Ok(())
			};

			match token_stack.last() {
				None => write_str()?,
				Some(PreprocEntry { write, .. }) => match write {
					WriteState::Skip => {},
					WriteState::Write => write_str()?,
					WriteState::Error(e) =>
						return Err(PreprocError {
							ty: PreprocErrorType::Other(e),
							span: SourceSpan {
								line: y,
								snip: line,
							},
						}),
				},
			}
		}
	}

	if let Some(last) = token_stack.pop() {
		Err(PreprocError {
			ty: PreprocErrorType::Other("Unterminated directive"),
			span: last.span,
		})
	} else {
		Ok(source_buffer)
	}
}
