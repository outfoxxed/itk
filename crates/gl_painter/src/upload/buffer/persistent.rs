// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{ffi::c_void, mem, ops::Range, ptr};

use gl::types::{GLenum, GLsizeiptr, GLsync, GLuint};

use super::GpuBuffer;

// FIXME: Performance seems a bit lower than it should be
// FIXME: Possible synchronization issues (gpu still reading as buffer is overwritten)
/// GPU buffer implemented with persistent mapped buffers
///
/// !Send to ensure the backing GL buffer is deleted
/// on the same thread
pub struct PersistentBuffer<T: bytemuck::AnyBitPattern> {
	buffer_type: GLenum,
	buffer: Vec<T>,
	mapped_region: *mut T,
	gl_buffer: GLuint,
	flush_region: Option<Range<usize>>,
	gl_buffer_size: usize,
	flush_fence: GLsync,
	backing_buffer_changed: bool,
	size_limit: usize,
}

// the raw pointer in `PersistentBuffer` sets !Send and !Sync
unsafe impl<T: bytemuck::AnyBitPattern> Sync for PersistentBuffer<T> {}

impl<T: bytemuck::AnyBitPattern> PersistentBuffer<T> {
	pub fn new(buffer_type: GLenum) -> Self {
		Self {
			buffer_type,
			buffer: Vec::new(),
			mapped_region: ptr::null_mut(),
			gl_buffer: 0,
			flush_region: None,
			gl_buffer_size: 0,
			flush_fence: ptr::null(),
			backing_buffer_changed: false,
			size_limit: 0,
		}
	}
}

impl<T: bytemuck::AnyBitPattern> GpuBuffer<T> for PersistentBuffer<T> {
	unsafe fn bind(&self) {
		gl::BindBuffer(self.buffer_type, self.gl_buffer);
	}

	unsafe fn prepare_write(&mut self) {
		self.sync_flush();
	}

	unsafe fn write(&mut self, offset: usize, data: &[T]) {
		if data.len() == 0 {
			return
		}

		let range = offset..offset + data.len();

		if let Some(slice) = self.buffer.get_mut(range.clone()) {
			slice.clone_from_slice(data);
			self.size_limit = usize::max(self.size_limit, range.end);

			if !self.mapped_region.is_null()
				&& self.flush_region.as_ref().map(|r| r.end).unwrap_or(0) <= range.end
				&& self.gl_buffer_size >= range.end
			{
				std::slice::from_raw_parts_mut::<T>(self.mapped_region, self.gl_buffer_size)
					[range.clone()]
				.clone_from_slice(data);
			}
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
		let buffer_len = self.buffer.len() * mem::size_of::<T>();

		// gl buffer size will be 0 if the buffer does not exist
		if self.gl_buffer_size < self.buffer.len() {
			if self.gl_buffer != 0 {
				gl::DeleteBuffers(1, &self.gl_buffer);
			}

			self.gl_buffer = 0;
			gl::GenBuffers(1, &mut self.gl_buffer);
			gl::BindBuffer(self.buffer_type, self.gl_buffer);

			gl::BufferStorage(
				self.buffer_type,
				buffer_len as GLsizeiptr,
				self.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
				gl::DYNAMIC_STORAGE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_WRITE_BIT,
			);

			self.mapped_region = gl::MapBufferRange(
				self.buffer_type,
				0,
				buffer_len as GLsizeiptr,
				gl::MAP_PERSISTENT_BIT | gl::MAP_WRITE_BIT | gl::MAP_FLUSH_EXPLICIT_BIT,
			) as *mut T;

			self.gl_buffer_size = self.buffer.len();
			self.backing_buffer_changed = true;

			if self.flush_fence != ptr::null() {
				gl::DeleteSync(self.flush_fence);
				self.flush_fence = ptr::null();
			}
		} else if let Some(range) = &self.flush_region {
			// flush region being `Some` means that range is not 0

			self.bind();

			gl::FlushMappedBufferRange(
				self.buffer_type,
				(range.start * mem::size_of::<T>()) as GLsizeiptr,
				(range.end * mem::size_of::<T>()) as GLsizeiptr,
			);
			gl::MemoryBarrier(gl::CLIENT_MAPPED_BUFFER_BARRIER_BIT);

			if self.flush_fence != ptr::null() {
				gl::DeleteSync(self.flush_fence);
			}
			self.flush_fence = gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
		}

		self.flush_region = None;
	}

	unsafe fn sync_flush(&mut self) {
		if !self.flush_fence.is_null() {
			gl::WaitSync(self.flush_fence, 0, u64::MAX);
			gl::DeleteSync(self.flush_fence);
			self.flush_fence = ptr::null();
		}
	}

	fn resize(&mut self, size: usize) {
		if size > self.buffer.len() {
			self.buffer.resize(size, T::zeroed());
		}
		self.size_limit = size;
	}

	fn len(&self) -> usize {
		self.size_limit
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

impl<T: bytemuck::AnyBitPattern> Drop for PersistentBuffer<T> {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &mut self.gl_buffer);
		}
	}
}
