// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::ops::Range;

use gl::types::{GLenum, GLuint};

pub mod compat;
pub mod persistent;

/// Buffer of data to upload to the GPU
///
/// # NOTES
/// `T`'s `drop` method will never be called.
pub trait GpuBuffer<T: bytemuck::AnyBitPattern> {
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
	unsafe fn write(&mut self) -> &mut CpuBacker<T>;
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
	fn backing_buffer(&self) -> GLuint;
}

/// Create a new GpuBuffer
///
/// does NOT bind new buffer
pub fn new<T: bytemuck::AnyBitPattern>(buffer_type: GLenum) -> Box<dyn GpuBuffer<T>> {
	match crate::check_gl_extensions().arb_buffer_storage {
		true => Box::new(persistent::PersistentBuffer::new(buffer_type)),
		false => Box::new(compat::CompatBuffer::new(buffer_type)),
	}
}

pub struct CpuBacker<T: bytemuck::AnyBitPattern> {
	buffer: Vec<T>,
	modified_range: Option<Range<usize>>,
	reallocated: bool,
}

impl<T: bytemuck::AnyBitPattern> CpuBacker<T> {
	pub fn new() -> Self {
		Self {
			buffer: Vec::new(),
			modified_range: None,
			reallocated: false,
		}
	}

	#[inline(always)]
	pub fn write(
		&mut self,
		offset: usize,
		data: impl IntoIterator<IntoIter = impl ExactSizeIterator<Item = T>>,
	) {
		let iter = data.into_iter();
		let count = iter.len();

		if offset + count > self.buffer.len() {
			self.buffer.resize_with(offset, || unreachable!());
			self.buffer.extend(iter);

			self.reallocated = true;
		} else {
			iter.enumerate().for_each(|(i, v)| self.buffer[offset + i] = v);
		}

		self.modified_range = Some(match &self.modified_range {
			Some(range) =>
				usize::min(range.start, offset)..usize::max(range.end, offset + count as usize),
			None => offset..offset + count,
		});
	}
}
