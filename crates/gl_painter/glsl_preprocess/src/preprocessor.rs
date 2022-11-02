// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{borrow::Cow, collections::HashMap, fmt::Debug, fs, io, path::Path, rc::Rc};

#[cfg(test)]
mod test;

#[derive(Clone)]
pub struct SourceSpan {
	line: LineId,
	snip: String,
}

#[derive(Debug, thiserror::Error)]
#[error("{span:?}: {ty}")]
pub struct PreprocError {
	ty: PreprocErrorType,
	span: SourceSpan,
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
	#[error(r#"could not include "{0}": {1:#}"#)]
	Include(String, io::Error),
	#[error("{0}")]
	Other(&'static str),
}

impl Debug for SourceSpan {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}: {}", self.line, self.snip)
	}
}

#[derive(Clone, PartialEq)]
pub struct LineId {
	file: Option<Rc<String>>,
	line: usize,
}

impl Debug for LineId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.file {
			Some(file) => write!(f, "{file}: {}", self.line),
			None => write!(f, "{}", self.line),
		}
	}
}

pub fn preprocess(
	source: &str,
	dir: &Path,
	mut defines: HashMap<String, String>,
) -> Result<(String, HashMap<usize, LineId>), PreprocError> {
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
		span: SourceSpan,
		token: PreprocToken<'a, 'd>,
		write: WriteState,
	}

	let mut source_buffer = String::with_capacity(source.len());
	let mut source_lines = 1;
	let mut token_stack = Vec::<PreprocEntry>::new();

	let mut line_buffer = {
		let source_lines = source.lines().count();

		source.lines()
			// `enumerate` dosen't play nice with `lines` and `rev`
			.rev()
			.enumerate()
			.map(|(i, l)| {
				(
					LineId {
						file: None,
						line: source_lines - i,
					},
					Cow::Borrowed(l)
				)
			})
			.collect::<Vec<_>>()
	};

	let mut line_mapping = HashMap::<usize, LineId>::with_capacity(line_buffer.len());

	while let Some((line_id, line)) = line_buffer.pop() {
		let line = &*line;

		if let Some('@') = line.trim_start().chars().next() {
			let span = || SourceSpan {
				line: line_id,
				snip: line.to_owned(),
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
								span: span(),
							}),
					};

					// get the match's target case from the defines list
					let target_case = match defines.get(condition) {
						Some(x) => x,
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::UndefinedTarget(condition.to_owned()),
								span: span(),
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
						span: span(),
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
								span: span(),
							}),
					};

					// split out cases in `a | b | c` form
					let cases = match args {
						Some(x) => x.split('|').into_iter().map(|c| c.trim()).collect::<Vec<_>>(),
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing case list"),
								span: span(),
							}),
					};

					// error if there are any duplicate cases, otherwise add them to hit list
					for case in &cases {
						if match_directive.hit_cases.iter().any(|h| h == case) {
							return Err(PreprocError {
								ty: PreprocErrorType::DuplicateCase(format!("{}", case)),
								span: span(),
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
								span: span(),
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
									span: span(),
								}),
						},
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing definition"),
								span: span(),
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
				"include" => {
					let (file, filename) = match args {
						Some(x) => {
							let path = dir.join(std::path::Path::new(x.trim()));

							(
								fs::read_to_string(&path).map_err(|e| PreprocError {
									ty: PreprocErrorType::Include(
										path.to_str().unwrap().to_owned(),
										e,
									),
									span: span(),
								})?,
								Rc::new(x.to_owned()),
							)
						},
						None =>
							return Err(PreprocError {
								ty: PreprocErrorType::Malformed("missing file to include"),
								span: span(),
							}),
					};

					let source_lines = file.lines().count();
					line_buffer.extend(file.lines().rev().enumerate().map(|(i, l)| {
						(
							LineId {
								file: Some(filename.clone()),
								line: source_lines - i,
							},
							Cow::Owned(l.to_owned()),
						)
					}));
				},
				_ =>
					return Err(PreprocError {
						ty: PreprocErrorType::UnknownDirective,
						span: span(),
					}),
			}
		} else {
			let mut write_str = || {
				if !line.is_empty() {
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
										line: line_id.clone(),
										snip: line.to_owned(),
									},
								}),
						};

						source_buffer.push_str(value);
						pi = sub.find('$');
					}

					source_buffer.push_str(sub);
					source_buffer.push('\n');
					line_mapping.insert(source_lines, line_id.clone());
					source_lines += 1;
				}

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
								line: line_id,
								snip: line.to_owned(),
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
		Ok((source_buffer, line_mapping))
	}
}
