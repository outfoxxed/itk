// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

pub use self::attribute::VertexAttribute;
use crate::{drawable::Drawable, shader::ShaderProgram};

pub mod attribute;
pub mod buffer;
pub mod compat;
pub mod ssbo;

pub trait Uploader<D: Drawable> {
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn prepare_write(&mut self);
	/// # SAFETY
	/// * must call prepare_write before calling write
	unsafe fn write(&mut self, drawable: &D);
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn begin_flush(&mut self);
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn sync_flush(&mut self);
	/// Bind input buffers and vertex attributes
	///
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn bind(&mut self);
	/// # SAFETY
	/// * must be called from GL thread, after sync_flush
	unsafe fn upload(&mut self);
	/// Should be called as soon as possible after
	/// finishing buffer use.
	///
	/// # SAFETY
	/// * must be called from GL thread
	unsafe fn finish_use(&mut self);
	/// Clear buffers
	unsafe fn clear(&mut self);
	fn shader_program(&self) -> &ShaderProgram;
}
