// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::ffi::c_void;

use gl::types::{GLenum, GLsizei, GLuint};

pub mod buffer;

pub struct VertexAttribute {
	pub ty: GLenum,
	pub count: usize,
	pub ty_size: usize,
}

pub struct Uploader<T: bytemuck::Pod> {
	vao: GLuint,
	vertex_buffer: Box<dyn buffer::GpuBuffer<T>>,
	index_buffer: Box<dyn buffer::GpuBuffer<GLuint>>,
	gl_type: GLenum,
	vertex_attributes: Vec<VertexAttribute>,
}

impl VertexAttribute {
	pub fn new<T>(count: usize, ty: GLenum) -> Self {
		VertexAttribute {
			ty,
			count,
			ty_size: std::mem::size_of::<T>(),
		}
	}
}

impl<T: bytemuck::Pod> Uploader<T> {
	/// Create new uploader
	///
	/// # SAFETY
	/// * must be called from GL thread
	pub unsafe fn new(gl_type: GLenum, vertex_attributes: Vec<VertexAttribute>) -> Self {
		let mut vao = 0;
		gl::GenVertexArrays(1, &mut vao);

		Self {
			vao,
			vertex_buffer: buffer::new::<T>(gl::ARRAY_BUFFER),
			index_buffer: buffer::new::<GLuint>(gl::ELEMENT_ARRAY_BUFFER),
			gl_type,
			vertex_attributes,
		}
	}

	/// Bind this uploader (VAO, VBO, EBO)
	///
	/// # SAFETY
	/// * must be called from GL thread
	pub unsafe fn bind(&mut self) {
		gl::BindVertexArray(self.vao);
		self.vertex_buffer.bind();
		self.index_buffer.bind();

		if self.vertex_buffer.has_backing_buffer()
			&& self.index_buffer.has_backing_buffer()
			&& (self.vertex_buffer.backing_buffer_changed()
				|| self.index_buffer.backing_buffer_changed())
		{
			Self::set_vertex_attributes(&self.vertex_attributes);
			self.vertex_buffer.clear_buffer_changed();
			self.index_buffer.clear_buffer_changed();
		}
	}

	/// Prepare uploader for writing
	///
	/// # SAFETY
	/// * must be called from GL thread
	pub unsafe fn prepare_write(&mut self) {
		self.vertex_buffer.prepare_write();
		self.index_buffer.prepare_write();
	}

	/// Write into this uploader
	///
	/// # SAFETY
	/// * must have previously called `prepare_write`
	pub unsafe fn write(&mut self) -> (buffer::BufferWriter<T>, buffer::BufferWriter<GLuint>) {
		(
			buffer::BufferWriter::new(&mut *self.vertex_buffer),
			buffer::BufferWriter::new(&mut *self.index_buffer),
		)
	}

	/// Begin flushing uploader buffers
	///
	/// # SAFETY
	/// * must be called from GL thread
	///
	/// # SIDE EFFECTS
	/// * may or may not bind VBO and EBO
	pub unsafe fn begin_flush(&mut self) {
		self.vertex_buffer.begin_flush();
		self.index_buffer.begin_flush();
	}

	/// Wait for buffer flushing to complete
	///
	/// # SAFETY
	/// * must be called from GL thread
	pub unsafe fn sync_flush(&mut self) {
		self.vertex_buffer.sync_flush();
		self.index_buffer.sync_flush();
	}

	/// TODO
	pub unsafe fn upload(&mut self) {
		self.bind();
		self.sync_flush();
		gl::DrawElements(
			self.gl_type,
			self.index_buffer.len() as GLsizei,
			gl::UNSIGNED_INT,
			0 as *const c_void,
		);
	}

	/// # SAFETY
	/// * VAO, VBO and EBO must be bound
	unsafe fn set_vertex_attributes(vertex_attributes: &[VertexAttribute]) {
		let stride =
			vertex_attributes.iter().map(|a| a.count * a.ty_size).sum::<usize>() as GLsizei;
		let mut offset = 0;

		for (i, attribute) in vertex_attributes.iter().enumerate() {
			gl::VertexAttribPointer(
				i as u32,
				attribute.count as GLsizei,
				attribute.ty,
				gl::FALSE,
				stride,
				offset as *const c_void,
			);

			gl::EnableVertexAttribArray(i as u32);

			offset += attribute.count * attribute.ty_size;
		}
	}
}
