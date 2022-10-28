// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{ffi::c_void, marker::PhantomData, mem, ops::Range, sync::MutexGuard};

use gl::types::{GLenum, GLsizeiptr, GLuint};

use super::GpuBuffer;

/// GPU buffer implemented with `glBufferSubData`
///
/// !Send to ensure the backing GL buffer is deleted
/// on the same thread
pub struct CompatBuffer<T: bytemuck::AnyBitPattern> {
	buffer_type: GLenum,
	buffer: Vec<T>,
	gl_buffer: GLuint,
	flush_region: Option<Range<usize>>,
	gl_buffer_size: usize,
	backing_buffer_changed: bool,
	_unsend: PhantomData<MutexGuard<'static, ()>>,
}

impl<T: bytemuck::AnyBitPattern> CompatBuffer<T> {
	pub fn new(buffer_type: GLenum) -> Self {
		Self {
			buffer_type,
			buffer: Vec::new(),
			gl_buffer: 0,
			flush_region: None,
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

	unsafe fn write(&mut self, offset: usize, data: &[T]) {
		if data.len() == 0 {
			return
		}

		let range = offset..offset + data.len();
		if let Some(slice) = self.buffer.get_mut(range.clone()) {
			slice.clone_from_slice(data);
		} else {
			self.resize(range.end);
			self.buffer[range.clone()].clone_from_slice(data);
		}

		self.flush_region = Some(match &self.flush_region {
			Some(old_range) =>
				usize::min(old_range.start, range.start)..usize::max(old_range.end, range.end),
			None => offset..offset + data.len(),
		});
	}

	unsafe fn begin_flush(&mut self) {
		let buffer_len = (self.buffer.len() * mem::size_of::<T>()) as GLsizeiptr;

		// gl buffer size will be 0 if the buffer does not exist
		if self.gl_buffer_size < self.buffer.len() {
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
				self.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
				gl::DYNAMIC_DRAW,
			);

			self.gl_buffer_size = self.buffer.len();
		} else if let Some(range) = &self.flush_region {
			// flush region being `Some` means that range is not 0

			self.bind();

			// TODO: invalidate buffer?
			gl::BufferSubData(
				self.buffer_type,
				(range.start * mem::size_of::<T>()) as GLsizeiptr,
				(range.end * mem::size_of::<T>()) as GLsizeiptr,
				self.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
			);
		}

		self.flush_region = None;
	}

	unsafe fn sync_flush(&mut self) {}

	fn resize(&mut self, size: usize) {
		self.buffer.resize(size, T::zeroed());
	}

	fn len(&self) -> usize {
		self.buffer.len()
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
