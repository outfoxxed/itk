// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{collections::HashMap, fs, io};

use proc_macro2::Span;
use syn::{
	braced,
	parse::{Parse, ParseStream},
	parse_macro_input,
	punctuated::Punctuated,
	Ident,
	Lit,
	LitStr,
	Token,
};

mod preprocessor;
mod validate;

struct PreprocessData {
	file: LitStr,
	ty: String,
	defines: HashMap<Ident, Ident>,
}

impl Parse for PreprocessData {
	// glsl_preprocess::preprocess_glsl! {
	//   shader: type "shader_file.glsl",
	//   defines: {
	//	   NAME: VAL,
	//	   NAME2: VAL2,
	//	 },
	// }
	fn parse(input: ParseStream) -> syn::Result<Self> {
		enum Entry {
			Shader(String, LitStr),
			Defines(HashMap<Ident, Ident>),
		}

		let entries =
			Punctuated::<(Span, Entry), Token![,]>::parse_terminated_with(input, |input| {
				let key = input.parse::<Ident>()?;

				input.parse::<Token![:]>()?;

				Ok((key.span(), match &key.to_string()[..] {
					// shader: "shader_file.glsl"
					"shader" => {
						let ty = input.parse::<Ident>()?;
						if ty != "vert" && ty != "frag" {
							Err(syn::Error::new(ty.span(), "Expected `vert` or `frag`"))
						} else {
							Ok(Entry::Shader(ty.to_string(), match input.parse::<Lit>()? {
								Lit::Str(x) => Ok(x),
								other => Err(syn::Error::new(other.span(), "Expected string")),
							}?))
						}
					},
					// define: { ... }
					"define" => {
						let braced;
						braced!(braced in input);

						// get `x: y` pairs
						let define_pairs =
							Punctuated::<(Ident, Ident), Token![,]>::parse_terminated_with(
								&braced,
								|input| {
									let key = input.parse::<Ident>()?;
									input.parse::<Token![:]>()?;
									let value = input.parse::<Ident>()?;

									Ok((key, value))
								},
							)?;

						let mut defines = HashMap::<Ident, Ident>::new();

						for (key, val) in define_pairs {
							match defines.get(&key) {
								Some(val) => Err(syn::Error::new(
									key.span(),
									format!(
										"{} is already defined as {}",
										key.to_string(),
										val.to_string()
									),
								)),
								None => {
									defines.insert(key, val);
									Ok(())
								},
							}?
						}

						Ok(Entry::Defines(defines))
					},
					_ => Err(syn::Error::new(key.span(), "Expected `shader` or `define`")),
				}?))
			})?;

		let mut shader = Option::<(String, LitStr)>::None;
		let mut defines = Option::<HashMap<Ident, Ident>>::None;

		for (key_span, entry) in entries {
			match entry {
				Entry::Shader(ty, s) => match shader.replace((ty, s)) {
					Some(_) => Err(syn::Error::new(key_span, "shader source already defined")),
					None => Ok(()),
				},
				Entry::Defines(x) => match defines.replace(x) {
					Some(_) => Err(syn::Error::new(key_span, "define block already defined")),
					None => Ok(()),
				},
			}?;
		}

		let (ty, shader) = match shader {
			Some(x) => Ok(x),
			None => Err(syn::Error::new(Span::call_site(), "missing shader source")),
		}?;

		Ok(PreprocessData {
			file: shader,
			ty,
			defines: match defines {
				Some(x) => Ok(x),
				None => Err(syn::Error::new(Span::call_site(), "missing define block")),
			}?,
		})
	}
}

#[proc_macro]
pub fn preprocess_glsl(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let preprocess_data = parse_macro_input!(tokens as PreprocessData);
	let manifest_dir = std::env::vars().find(|(key, _)| key == "CARGO_MANIFEST_DIR").unwrap().1;
	let filepath = std::path::Path::new(&manifest_dir).join(&preprocess_data.file.value());

	// closure to use the ? operator
	let r = (|| -> Result<proc_macro::TokenStream, syn::Error> {
		let shader_source = match fs::read_to_string(&filepath) {
			Ok(x) => Ok(x),
			Err(e) => match e.kind() {
				io::ErrorKind::NotFound => Err(syn::Error::new(
					preprocess_data.file.span(),
					format!(
						"Shader source file not found (paths are relative to cargo manifest): {filepath:?}"
					),
				)),
				e => Err(syn::Error::new(
					preprocess_data.file.span(),
					format!("Could not read shader source: {e:#}"),
				)),
			},
		}?;

		let defines = preprocess_data
			.defines
			.into_iter()
			.map(|(k, v)| (k.to_string(), v.to_string()))
			.collect::<HashMap<String, String>>();

		let (src, line_mapping) =
			preprocessor::preprocess(&shader_source, filepath.parent().unwrap(), defines).map_err(
				|e| syn::Error::new(Span::call_site(), format!("error in shader: {e:#}")),
			)?;

		validate::validate_shader(&src, &preprocess_data.ty, &line_mapping).map_err(|e| {
			syn::Error::new(
				preprocess_data.file.span(),
				format!("error(s) during shader validation:\n{e}"),
			)
		})?;

		Ok(format!("{src:?}").parse().unwrap())
	})();

	match r {
		Ok(x) => x,
		Err(e) => proc_macro::TokenStream::from(e.to_compile_error()),
	}
}
