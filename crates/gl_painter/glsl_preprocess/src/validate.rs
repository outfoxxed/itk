// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{collections::HashMap, io::Write};

use crate::preprocessor::LineId;

pub fn validate_shader(
	source: &str,
	ty: &str,
	line_mapping: &HashMap<usize, LineId>,
) -> Result<(), String> {
	let validator = std::env::vars()
		.find(|(key, _)| key == "GLSL_VALIDATOR")
		.map(|(_, v)| v)
		.unwrap_or("glslangValidator".to_owned());

	let mut file = tempfile::NamedTempFile::new().unwrap();
	file.write(source.as_bytes()).unwrap();

	let stdout = match std::process::Command::new(&validator)
		.args(["-S", ty, file.path().to_str().unwrap()])
		.output()
	{
		Ok(x) => x.stdout,
		Err(e) => {
			eprintln!(r#"WARNING: Could not run GLSL validator "{validator}" {e:#}"#);
			return Ok(())
		},
	};

	let mut log = std::str::from_utf8(&stdout).unwrap().trim();
	// cut off the first line, which says what file the error came from
	log = log.split_once('\n').map(|(_, log)| log).unwrap_or(log);

	// match out the line numbers and reformat errors using the line mapping
	let log = log
		.lines()
		.filter(|line| !line.contains("compilation errors"))
		.map(|mut line| {
			line = line.strip_prefix("ERROR: ").unwrap();
			line = line.split_once(':').map(|(_, x)| x).unwrap();
			let (line_num, message) = line.split_once(':').unwrap();
			let line_id = line_mapping.get(&line_num.parse::<usize>().unwrap()).unwrap();

			format!("{line_id:?}: {message}")
		})
		.collect::<String>();

	if log.is_empty() {
		Ok(())
	} else {
		Err(log.to_owned())
	}
}
