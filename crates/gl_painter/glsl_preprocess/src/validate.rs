use std::io::Write;

pub fn validate_shader(source: &str, ty: &str) -> Result<(), String> {
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

	if log.is_empty() {
		Ok(())
	} else {
		Err(log.to_owned())
	}
}
