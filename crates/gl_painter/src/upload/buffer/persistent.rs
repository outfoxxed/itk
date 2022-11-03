// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::{ffi::c_void, mem, ptr};

use gl::types::{GLenum, GLsizeiptr, GLsync, GLuint};

use super::{CpuBacker, GpuBuffer};

// FIXME: Performance seems a bit lower than it should be
// FIXME: Possible synchronization issues (gpu still reading as buffer is overwritten)
/// GPU buffer implemented with persistent mapped buffers
///
/// !Send to ensure the backing GL buffer is deleted
/// on the same thread
pub struct PersistentBuffer<T: bytemuck::AnyBitPattern> {
	buffer_type: GLenum,
	backer: CpuBacker<T>,
	mapped_region: *mut T,
	gl_buffer: GLuint,
	gl_buffer_size: usize,
	flush_fence: GLsync,
	backing_buffer_changed: bool,
}

// the raw pointer in `PersistentBuffer` sets !Send and !Sync
unsafe impl<T: bytemuck::AnyBitPattern> Sync for PersistentBuffer<T> {}

impl<T: bytemuck::AnyBitPattern> PersistentBuffer<T> {
	pub fn new(buffer_type: GLenum) -> Self {
		Self {
			buffer_type,
			backer: CpuBacker::new(),
			mapped_region: ptr::null_mut(),
			gl_buffer: 0,
			gl_buffer_size: 0,
			flush_fence: ptr::null(),
			backing_buffer_changed: false,
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

	unsafe fn write(&mut self) -> &mut CpuBacker<T> {
		&mut self.backer
	}

	unsafe fn begin_flush(&mut self) {
		let buffer_len = self.backer.buffer.len() * mem::size_of::<T>();

		// gl buffer size will be 0 if the buffer does not exist
		if self.gl_buffer_size < self.backer.buffer.len() {
			if self.gl_buffer != 0 {
				gl::DeleteBuffers(1, &self.gl_buffer);
			}

			self.gl_buffer = 0;
			gl::GenBuffers(1, &mut self.gl_buffer);
			gl::BindBuffer(self.buffer_type, self.gl_buffer);

			gl::BufferStorage(
				self.buffer_type,
				buffer_len as GLsizeiptr,
				self.backer.buffer[..].as_ptr() as *const _ as *const c_void,
				//bytemuck::cast_slice::<T, u8>(&self.buffer).as_ptr() as *const c_void,
				gl::DYNAMIC_STORAGE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_WRITE_BIT,
			);

			self.mapped_region = gl::MapBufferRange(
				self.buffer_type,
				0,
				buffer_len as GLsizeiptr,
				gl::MAP_PERSISTENT_BIT | gl::MAP_WRITE_BIT | gl::MAP_FLUSH_EXPLICIT_BIT,
			) as *mut T;

			self.gl_buffer_size = self.backer.buffer.len();
			self.backing_buffer_changed = true;

			if self.flush_fence != ptr::null() {
				gl::DeleteSync(self.flush_fence);
				self.flush_fence = ptr::null();
			}
		} else if let Some(range) = &self.backer.modified_range {
			// flush region being `Some` means that range is not 0

			self.bind();

			std::slice::from_raw_parts_mut::<T>(self.mapped_region, self.gl_buffer_size)
				[self.backer.modified_range.as_ref().unwrap().clone()]
			.clone_from_slice(&self.backer.buffer[range.clone()]);

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

		self.backer.modified_range = None;
	}

	unsafe fn sync_flush(&mut self) {
		if !self.flush_fence.is_null() {
			gl::WaitSync(self.flush_fence, 0, u64::MAX);
			gl::DeleteSync(self.flush_fence);
			self.flush_fence = ptr::null();
		}
	}

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

impl<T: bytemuck::AnyBitPattern> Drop for PersistentBuffer<T> {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteBuffers(1, &mut self.gl_buffer);
		}
	}
}
