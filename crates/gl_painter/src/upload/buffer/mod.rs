// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::mem::MaybeUninit;

use gl::types::GLenum;

pub mod compat;
pub mod persistent;

/// Buffer of data to upload to the GPU
///
/// # NOTES
/// `T`'s `drop` method will never be called.
pub trait GpuBuffer<T: bytemuck::Pod> {
	/// Bind this buffer
	///
	/// if this buffer has no backing buffer, the 0 buffer
	/// will be bound
	///
	/// # SAFETY
	/// * must be called from GL thread
	/// * must be flushed
	unsafe fn bind(&self);
	/// Prepare for writing
	///
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn prepare_write(&mut self);
	/// Write into this buffer
	///
	/// # SAFETY
	/// * must have previously called `prepare_write`
	unsafe fn write(&mut self, offset: usize, data: &[MaybeUninit<T>]);
	/// Begin flushing this buffer. Call `ready` to wait for completion.
	///
	/// # SAFETY
	/// * must be called from GL thread
	///
	///	# SIDE EFFECTS
	///	* may or may not bind this buffer
	unsafe fn begin_flush(&mut self);
	/// Wait for flush to complete
	///
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn sync_flush(&mut self);
	/// Resize this buffer
	///
	/// if new size is smaller than current size,
	/// only elements fitting within the new size will stay
	///
	/// backing buffer's size will only be updated when `begin_flush` is called
	fn resize(&mut self, size: usize);
	/// Get the length of this buffer
	fn len(&self) -> usize;
	/// Check if the underlying buffer exists
	fn has_backing_buffer(&self) -> bool;
	/// Check if the underlying buffer has been changed since
	/// it was last cleared
	fn backing_buffer_changed(&self) -> bool;
	/// Clear buffer changed flag
	fn clear_buffer_changed(&mut self);
}

/// Create a new GpuBuffer
///
/// does NOT bind new buffer
pub fn new<T: bytemuck::Pod>(buffer_type: GLenum) -> Box<dyn GpuBuffer<T>> {
	match crate::check_gl_extensions().arb_buffer_storage {
		true => Box::new(persistent::PersistentBuffer::new(buffer_type)),
		false => Box::new(compat::CompatBuffer::new(buffer_type)),
	}
}

pub struct BufferWriter<'b, T: bytemuck::Pod> {
	buffer: &'b mut dyn GpuBuffer<T>,
}

impl<'b, T: bytemuck::Pod> BufferWriter<'b, T> {
	/// SAFETY
	/// * buffer must be valid to write to for 'b
	pub unsafe fn new(buffer: &'b mut dyn GpuBuffer<T>) -> Self {
		Self { buffer }
	}

	#[inline]
	pub fn write(&mut self, offset: usize, data: &[MaybeUninit<T>]) {
		unsafe { self.buffer.write(offset, data) };
	}

	#[inline]
	pub fn resize(&mut self, size: usize) {
		self.buffer.resize(size);
	}
}
