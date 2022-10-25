use std::ffi::c_void;

use gl::types::{GLsizei, GLuint};

use super::{
	attribute::GLtype,
	buffer::{self, GpuBuffer},
	Uploader,
};
use crate::{
	drawable::{Drawable, VertexPassable},
	shader::ShaderProgram,
	upload::VertexAttribute,
};

pub struct SsboUploader<D: Drawable> {
	vao: GLuint,
	vertex_buffer: Box<dyn GpuBuffer<SsboVertex<D::Vertex>>>,
	index_buffer: Box<dyn GpuBuffer<u32>>,
	storage_buffer: Box<dyn GpuBuffer<D::DrawableData>>,
	shader: ShaderProgram,
}

#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
struct SsboVertex<V: bytemuck::Pod> {
	vertex: V,
	ssbo_index: u32,
}

impl<V: bytemuck::Pod> Copy for SsboVertex<V> {}

impl<V: bytemuck::Pod> core::clone::Clone for SsboVertex<V> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<D: Drawable> SsboUploader<D> {
	/// # SAFETY
	/// * must be called from GL thread (shader creation)
	pub unsafe fn new() -> Self {
		Self {
			vao: 0,
			vertex_buffer: buffer::new::<SsboVertex<D::Vertex>>(gl::ARRAY_BUFFER),
			index_buffer: buffer::new::<u32>(gl::ELEMENT_ARRAY_BUFFER),
			storage_buffer: buffer::new::<D::DrawableData>(gl::SHADER_STORAGE_BUFFER),
			shader: D::SHADER_SOURCE.create_program(true),
		}
	}

	/// # SAFETY
	/// * VAO, VBO and EBO must be bound
	unsafe fn set_vertex_attributes() {
		const SSBO_ATTRIBUTE: &[VertexAttribute] = &[VertexAttribute::new::<u32>(1)];
		let stride = D::Vertex::VERTEX_ATTRIBUTES
			.iter()
			.chain(SSBO_ATTRIBUTE)
			.map(|a| a.count * a.ty_size)
			.sum::<usize>() as GLsizei;

		let mut offset = 0;
		for (i, attribute) in D::Vertex::VERTEX_ATTRIBUTES.iter().chain(SSBO_ATTRIBUTE).enumerate()
		{
			if attribute.is_integer {
				gl::VertexAttribIPointer(
					i as u32,
					attribute.count as GLsizei,
					attribute.ty,
					stride,
					offset as *const c_void,
				);
			} else {
				gl::VertexAttribPointer(
					i as u32,
					attribute.count as GLsizei,
					attribute.ty,
					gl::FALSE,
					stride,
					offset as *const c_void,
				);
			}

			gl::EnableVertexAttribArray(i as u32);

			offset += attribute.count * attribute.ty_size;
		}
	}
}

impl<D: Drawable> Uploader<D> for SsboUploader<D> {
	unsafe fn prepare_write(&mut self) {
		self.vertex_buffer.prepare_write();
		self.index_buffer.prepare_write();
		self.storage_buffer.prepare_write();
	}

	unsafe fn write(&mut self, drawable: D) {
		let drawable_data = drawable.drawable_data();

		// FIXME: excess allocation, move this somewhere else
		// and/or make buffers accept iterators
		let mut vertex_data = Vec::new();
		let mut index_data = Vec::new();

		drawable.drawable_vertices(&mut vertex_data, &mut index_data);

		let combined_vertex_data = vertex_data
			.into_iter()
			.map(|vertex| SsboVertex {
				vertex,
				ssbo_index: self.storage_buffer.len() as u32,
			})
			.collect::<Vec<_>>();

		let vbo_index = self.vertex_buffer.len();
		index_data.iter_mut().for_each(|i| *i = vbo_index as u32 + *i);

		self.vertex_buffer.write(vbo_index, &combined_vertex_data);
		self.index_buffer.write(self.index_buffer.len(), &index_data);
		self.storage_buffer.write(self.storage_buffer.len(), &[drawable_data]);
	}

	unsafe fn begin_flush(&mut self) {
		self.vertex_buffer.begin_flush();
		self.index_buffer.begin_flush();
		self.storage_buffer.begin_flush();
	}

	unsafe fn sync_flush(&mut self) {
		self.vertex_buffer.sync_flush();
		self.index_buffer.sync_flush();
		self.storage_buffer.sync_flush();
	}

	unsafe fn bind(&mut self) {
		if !(self.vertex_buffer.has_backing_buffer()
			&& self.index_buffer.has_backing_buffer()
			&& self.storage_buffer.has_backing_buffer())
		{
			return
		}

		let no_vbo = self.vao == 0;
		if no_vbo {
			gl::GenVertexArrays(1, &mut self.vao);
		}
		gl::BindVertexArray(self.vao);

		if no_vbo
			|| self.vertex_buffer.backing_buffer_changed()
			|| self.index_buffer.backing_buffer_changed()
			|| self.storage_buffer.backing_buffer_changed()
		{
			Self::set_vertex_attributes();
			self.vertex_buffer.clear_buffer_changed();
			self.index_buffer.clear_buffer_changed();
			self.storage_buffer.backing_buffer_changed();
		}

		self.vertex_buffer.bind();
		self.index_buffer.bind();
		self.storage_buffer.bind();
	}

	unsafe fn upload(&mut self) {
		self.shader.bind();
		self.bind();
		self.sync_flush();
		gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, 0, self.storage_buffer.backing_buffer());
		gl::DrawElements(
			D::GL_TYPE,
			self.index_buffer.len() as GLsizei,
			u32::GL_TYPE,
			0 as *const c_void,
		);
		self.finish_use();
	}

	unsafe fn finish_use(&mut self) {
		// TODO: synchronization
	}

	unsafe fn clear(&mut self) {
		self.vertex_buffer.resize(0);
		self.index_buffer.resize(0);
		self.storage_buffer.resize(0);
	}
}
