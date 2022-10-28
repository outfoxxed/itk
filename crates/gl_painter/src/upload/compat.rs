// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::ffi::c_void;

use gl::types::{GLsizei, GLuint};

use super::{
	attribute::GLtype,
	buffer::{self, GpuBuffer},
	Uploader,
};
use crate::{
	drawable::{Drawable, DrawableData, VertexPassable},
	shader::ShaderProgram,
};

pub struct CompatUploader<D: Drawable> {
	vao: GLuint,
	vertex_buffer: Box<
		dyn GpuBuffer<
			CompatVertex<
				<D::Drawable as DrawableData>::Compat,
				<D::Vertex as DrawableData>::Compat,
			>,
		>,
	>,
	index_buffer: Box<dyn GpuBuffer<u32>>,
	shader: ShaderProgram,
}

#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
struct CompatVertex<D: bytemuck::Pod, V: bytemuck::Pod> {
	vertex: V,
	drawable_data: D,
}

impl<D: bytemuck::Pod, V: bytemuck::Pod> Copy for CompatVertex<D, V> {}

impl<D: bytemuck::Pod, V: bytemuck::Pod> core::clone::Clone for CompatVertex<D, V> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<D: Drawable> CompatUploader<D> {
	/// # SAFETY
	/// * must be called from GL thread (shader creation)
	pub unsafe fn new() -> Self {
		Self {
			vao: 0,
			vertex_buffer: buffer::new::<
				CompatVertex<
					<D::Drawable as DrawableData>::Compat,
					<D::Vertex as DrawableData>::Compat,
				>,
			>(gl::ARRAY_BUFFER),
			index_buffer: buffer::new::<u32>(gl::ELEMENT_ARRAY_BUFFER),
			shader: D::SHADER_SOURCE.create_program(false),
		}
	}

	/// # SAFETY
	/// * VAO, VBO and EBO must be bound
	unsafe fn set_vertex_attributes() {
		let stride = <D::Vertex as DrawableData>::Compat::VERTEX_ATTRIBUTES
			.iter()
			.chain(<D::Drawable as DrawableData>::Compat::VERTEX_ATTRIBUTES.iter())
			.map(|a| a.padding + a.count * a.ty_size)
			.sum::<usize>() as GLsizei;

		let mut offset = 0;
		for (i, attribute) in <D::Vertex as DrawableData>::Compat::VERTEX_ATTRIBUTES
			.iter()
			.chain(<D::Drawable as DrawableData>::Compat::VERTEX_ATTRIBUTES.iter())
			.enumerate()
		{
			offset += attribute.padding;

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

impl<D: Drawable> Uploader<D> for CompatUploader<D> {
	unsafe fn prepare_write(&mut self) {
		self.vertex_buffer.prepare_write();
		self.index_buffer.prepare_write();
	}

	unsafe fn write(&mut self, drawable: &D) {
		let drawable_data = drawable.drawable_data().into_compat();

		// FIXME: excess allocation, move this somewhere else
		// and/or make buffers accept iterators
		let mut vertex_data = Vec::new();
		let mut index_data = Vec::new();

		drawable.drawable_vertices(&mut vertex_data, &mut index_data);

		let combined_vertex_data = vertex_data
			.into_iter()
			.map(|vertex| CompatVertex {
				vertex: vertex.into_compat(),
				drawable_data: drawable_data.clone(),
			})
			.collect::<Vec<_>>();

		let vbo_index = self.vertex_buffer.len();
		index_data.iter_mut().for_each(|i| *i = vbo_index as u32 + *i);

		self.vertex_buffer.write(vbo_index, &combined_vertex_data);
		self.index_buffer.write(self.index_buffer.len(), &index_data);
	}

	unsafe fn begin_flush(&mut self) {
		self.vertex_buffer.begin_flush();
		self.index_buffer.begin_flush();
	}

	unsafe fn sync_flush(&mut self) {
		self.vertex_buffer.sync_flush();
		self.index_buffer.sync_flush();
	}

	unsafe fn bind(&mut self) {
		if !(self.vertex_buffer.has_backing_buffer() && self.index_buffer.has_backing_buffer()) {
			return
		}

		let no_vao = self.vao == 0;
		if no_vao {
			gl::GenVertexArrays(1, &mut self.vao);
		}
		gl::BindVertexArray(self.vao);

		self.vertex_buffer.bind();
		self.index_buffer.bind();

		if no_vao
			|| self.vertex_buffer.backing_buffer_changed()
			|| self.index_buffer.backing_buffer_changed()
		{
			Self::set_vertex_attributes();
			self.vertex_buffer.clear_buffer_changed();
			self.index_buffer.clear_buffer_changed();
		}
	}

	unsafe fn upload(&mut self) {
		self.shader.bind();
		self.bind();
		self.sync_flush();
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
	}

	fn shader_program(&self) -> &ShaderProgram {
		&self.shader
	}
}
