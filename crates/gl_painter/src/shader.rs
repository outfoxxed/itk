// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use gl::types::{GLenum, GLint, GLuint};
use thiserror::Error;

pub enum ShaderType {
	Vertex,
	Fragment,
}

pub struct Shader {
	ty: ShaderType,
	shader_object: GLuint,
}

pub struct ShaderProgram {
	pub program_object: GLuint,
}

impl ShaderType {
	#[inline]
	pub fn gl_type(&self) -> GLenum {
		match self {
			Self::Vertex => gl::VERTEX_SHADER,
			Self::Fragment => gl::FRAGMENT_SHADER,
		}
	}
}

#[derive(Debug, Error)]
pub enum ShaderCompileError {
	#[error("could not create shader (glCreateShader returned 0)")]
	CouldNotCreate,
	#[error("could not compile shader - driver log:\n{0}\n")]
	Compile(String),
}

#[derive(Debug, Error)]
pub enum ShaderLinkError {
	#[error("expected a vertex shader and a fragment shader")]
	InvalidShader,
	#[error("could not create program (glCreateProgram returned 0)")]
	CouldNotCreate,
	#[error("could not link shader - driver log:\n{0}\n")]
	Link(String),
}

impl Shader {
	pub fn compile(ty: ShaderType, source: &str) -> Result<Self, ShaderCompileError> {
		unsafe {
			let shader = gl::CreateShader(ty.gl_type());
			if shader == 0 {
				return Err(ShaderCompileError::CouldNotCreate)
			}

			let src_ptr = source.as_bytes().as_ptr() as *const i8;
			let len = source.len() as GLint;
			gl::ShaderSource(shader, 1, &src_ptr, &len);

			gl::CompileShader(shader);

			let mut compile_status = 0 as GLint;
			gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut compile_status);

			if compile_status != gl::TRUE as i32 {
				let mut log_length = 0 as GLint;
				gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);

				let mut log = Vec::<u8>::with_capacity(log_length as usize);
				gl::GetShaderInfoLog(
					shader,
					log_length,
					&mut log_length,
					log.as_mut_ptr() as *mut i8,
				);
				// glGetShaderInfoLog always writes a null terminator. Subtracting one removes it.
				log.set_len((log_length - 1) as usize);

				#[rustfmt::skip]
				return Err(ShaderCompileError::Compile(
					// The OpenGL driver should not be returning invalid utf8,
					// and the shader's source can't be invalid utf8 either,
					// due to it being an &str.
					String::from_utf8(log)
						.expect("OpenGL driver returned invalid utf8 string while reading shader info log")
				));
			}

			Ok(Shader {
				shader_object: shader,
				ty,
			})
		}
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		// Reduces refcount for shader.
		// The OpenGL driver will only delete the backing shader object
		// when it is not attached to shader program.
		unsafe { gl::DeleteShader(self.shader_object) };
	}
}

impl ShaderProgram {
	pub fn link(vertex_shader: &Shader, fragment_shader: &Shader) -> Result<Self, ShaderLinkError> {
		match (&vertex_shader.ty, &fragment_shader.ty) {
			(ShaderType::Vertex, ShaderType::Fragment) => {},
			_ => return Err(ShaderLinkError::InvalidShader),
		}

		unsafe {
			let program = gl::CreateProgram();
			if program == 0 {
				return Err(ShaderLinkError::CouldNotCreate)
			}

			gl::AttachShader(program, vertex_shader.shader_object);
			gl::AttachShader(program, fragment_shader.shader_object);

			gl::LinkProgram(program);

			// allows earlier deletion of shader objects.
			gl::DetachShader(program, vertex_shader.shader_object);
			gl::DetachShader(program, fragment_shader.shader_object);

			let mut link_status = 0;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut link_status);

			if link_status != gl::TRUE as i32 {
				let mut log_length = 0 as GLint;
				gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);

				let mut log = Vec::<u8>::with_capacity(log_length as usize);
				gl::GetProgramInfoLog(
					program,
					log_length,
					&mut log_length,
					log.as_mut_ptr() as *mut i8,
				);
				// glGetProgramInfoLog always writes a null terminator. Subtracting one removes it.
				log.set_len((log_length - 1) as usize);

				#[rustfmt::skip]
				return Err(ShaderLinkError::Link(
					// The OpenGL driver should not be returning invalid utf8.
					String::from_utf8(log)
						.expect("OpenGL driver returned invalid utf8 string while reading program info log")
				));
			}

			Ok(ShaderProgram {
				program_object: program,
			})
		}
	}

	pub fn bind(&self) {
		unsafe { gl::UseProgram(self.program_object) };
	}
}

impl Drop for ShaderProgram {
	fn drop(&mut self) {
		// Reduces refcount for shader program.
		// The OpenGL driver will only delete the backing shader program
		// when it is not part of any renderin context.
		unsafe { gl::DeleteProgram(self.program_object) };
	}
}
