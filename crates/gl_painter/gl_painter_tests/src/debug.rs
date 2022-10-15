// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::ffi::{c_char, c_void, CStr};

use gl::types::{GLenum, GLsizei, GLuint};

pub fn setup_gl_debug() {
	unsafe {
		gl::Enable(gl::DEBUG_OUTPUT);
		gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
		gl::DebugMessageCallback(Some(gl_debug), std::ptr::null());
		gl::DebugMessageControl(
			gl::DONT_CARE,
			gl::DONT_CARE,
			gl::DONT_CARE,
			0,
			std::ptr::null(),
			gl::TRUE,
		);
	}
}

extern "system" fn gl_debug(
	source: GLenum,
	ty: GLenum,
	_id: GLuint,
	severity: GLenum,
	_length: GLsizei,
	message: *const c_char,
	_user_param: *mut c_void,
) {
	let message = unsafe { CStr::from_ptr(message).to_str().unwrap() };
	let source = match source {
		gl::DEBUG_SOURCE_API => "API",
		gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
		gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
		gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
		gl::DEBUG_SOURCE_APPLICATION => "Application",
		gl::DEBUG_TYPE_OTHER => "Other",
		_ => unreachable!(),
	};

	let ty = match ty {
		gl::DEBUG_TYPE_ERROR => "Error",
		gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
		gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
		gl::DEBUG_TYPE_PORTABILITY => "Portability",
		gl::DEBUG_TYPE_PERFORMANCE => "Performance",
		gl::DEBUG_TYPE_MARKER => "Marker",
		gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
		gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
		gl::DEBUG_TYPE_OTHER => "Other",
		_ => unreachable!(),
	};

	match severity {
		gl::DEBUG_SEVERITY_HIGH => log::error!(target: "OpenGL", "{ty}: {source}: {message}"),
		gl::DEBUG_SEVERITY_MEDIUM => log::warn!(target: "OpenGL", "{ty}: {source}: {message}"),
		gl::DEBUG_SEVERITY_LOW => log::debug!(target: "OpenGL", "{ty}: {source}: {message}"),
		gl::DEBUG_SEVERITY_NOTIFICATION =>
			log::trace!(target: "OpenGL", "{ty}: {source}: {message}"),
		_ => unreachable!(),
	}
}
