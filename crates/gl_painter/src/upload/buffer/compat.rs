// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{ffi::c_void, marker::PhantomData, mem, sync::MutexGuard};

use gl::types::{GLenum, GLsizeiptr, GLuint};

use super::{CpuBacker, GpuBuffer};

/// GPU buffer implemented with `glBufferSubData`
///
/// !Send to ensure the backing GL buffer is deleted
/// on the same thread
pub struct CompatBuffer<T: bytemuck::AnyBitPattern> {
	buffer_type: GLenum,
	backer: CpuBacker<T>,
	gl_buffer: GLuint,
	gl_buffer_size: usize,
	backing_buffer_changed: bool,
	_unsend: PhantomData<MutexGuard<'static, ()>>,
}

impl<T: bytemuck::AnyBitPattern> CompatBuffer<T> {
	pub fn new(buffer_type: GLenum) -> Self {
		Self {
			buffer_type,
			backer: CpuBacker::new(),
			gl_buffer: 0,
			gl_buffer_size: 0,
			backing_buffer_changed: false,
			_unsend: PhantomData,
		}
	}
}

impl<T: bytemuck::AnyBitPattern> GpuBuffer<T> for CompatBuffer<T> {
	unsafe fn bind(&self) {
		gl::BindBuffer(self.buffer_type, self.gl_buffer);
	}

	unsafe fn prepare_write(&mut self) {}

	unsafe fn write(&mut self) -> &mut CpuBacker<T> {
		&mut self.backer
	}

	unsafe fn begin_flush(&mut self) {
		let buffer_len = (self.backer.buffer.len() * mem::size_of::<T>()) as GLsizeiptr;

		// gl buffer size will be 0 if the buffer does not exist
		if self.gl_buffer_size < self.backer.buffer.len() {
			if self.gl_buffer == 0 {
				gl::GenBuffers(1, &mut self.gl_buffer);
				gl::BindBuffer(self.buffer_type, self.gl_buffer);

				self.backing_buffer_changed = true;
			} else {
				self.bind();
			}

			gl::BufferData(
				self.buffer_type,
				buffer_len,
				self.backer.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
				gl::DYNAMIC_DRAW,
			);

			self.gl_buffer_size = self.backer.buffer.len();
		} else if let Some(range) = &self.backer.modified_range {
			// flush region being `Some` means that range is not 0

			self.bind();

			// TODO: invalidate buffer?
			gl::BufferSubData(
				self.buffer_type,
				(range.start * mem::size_of::<T>()) as GLsizeiptr,
				(range.end * mem::size_of::<T>()) as GLsizeiptr,
				self.backer.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
			);
		}

		self.backer.modified_range = None;
	}

	unsafe fn sync_flush(&mut self) {}

	fn resize(&mut self, size: usize) {
		self.backer.buffer.resize(size, T::zeroed());
	}

	fn len(&self) -> usize {
		self.backer.buffer.len()
	}

	fn has_backing_buffer(&self) -> bool {
		self.gl_buffer != 0
	}

	fn backing_buffer_changed(&self) -> bool {
		self.backing_buffer_changed
	}

	fn clear_buffer_changed(&mut self) {
		self.backing_buffer_changed = false;
	}

	fn backing_buffer(&self) -> GLuint {
		self.gl_buffer
	}
}

impl<T: bytemuck::AnyBitPattern> Drop for CompatBuffer<T> {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &mut self.gl_buffer);
		}
	}
}
