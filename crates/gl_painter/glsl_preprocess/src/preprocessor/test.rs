use std::{collections::HashMap, fs, path::Path, rc::Rc};

use super::preprocess;
use crate::preprocessor::LineId;

#[test]
fn test_preprocessor() {
	let (text, line_map) = preprocess(
		&fs::read_to_string("src/preprocessor/test/test_preprocessor.glsl").unwrap(),
		Path::new("src/preprocessor/test"),
		HashMap::new(),
	)
	.unwrap();

	assert_eq!(text, fs::read_to_string("src/preprocessor/test/test.glsl.results").unwrap());

	let inc_file = Rc::new("include.glsl".to_string());

	#[rustfmt::skip]
	assert_eq!(line_map, HashMap::from([
		(1, LineId { line: 1, file: Some(inc_file.clone()) }),
		(2, LineId { line: 2, file: Some(inc_file.clone()) }),
		(3, LineId { line: 4, file: Some(inc_file.clone()) }),
		(4, LineId { line: 6, file: Some(inc_file.clone()) }),
		(5, LineId { line: 10, file: Some(inc_file.clone()) }),
		(6, LineId { line: 11, file: Some(inc_file.clone()) }),
		(7, LineId { line: 4, file: None }),
		(8, LineId { line: 6, file: None }),
		(9, LineId { line: 1, file: Some(inc_file.clone()) }),
		(10, LineId { line: 2, file: Some(inc_file.clone()) }),
		(11, LineId { line: 4, file: Some(inc_file.clone()) }),
		(12, LineId { line: 6, file: Some(inc_file.clone()) }),
		(13, LineId { line: 13, file: Some(inc_file.clone()) }),
		(14, LineId { line: 1, file: Some(inc_file.clone()) }),
		(15, LineId { line: 2, file: Some(inc_file.clone()) }),
		(16, LineId { line: 4, file: Some(inc_file.clone()) }),
		(17, LineId { line: 6, file: Some(inc_file.clone()) }),
		(18, LineId { line: 15, file: Some(inc_file.clone()) }),
		(19, LineId { line: 13, file: None }),
	]));
}
