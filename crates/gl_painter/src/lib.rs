// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

#![allow(incomplete_features)]
// Allows use of `-> impl` in traits.
// Can be desugared if nessesary to support stable,
// if it hasn't come to stable by the time ITK is usable.
#![feature(return_position_impl_trait_in_trait)]

use std::ffi::CStr;

pub mod shader;
pub mod upload;

#[derive(Debug)]
pub struct GlExtensions {
	arb_buffer_storage: bool,
}

// TODO: cache this in some way where used
pub fn check_gl_extensions() -> GlExtensions {
	let mut extension_count = 0;
	unsafe { gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut extension_count) };

	let extensions = (0..extension_count)
		.map(|i| {
			unsafe { CStr::from_ptr(gl::GetStringi(gl::EXTENSIONS, i as u32) as *const i8) }
				.to_str()
				.expect("OpenGL driver returned a non UTF8 string when requesting extensions")
		})
		.collect::<Vec<_>>();

	GlExtensions {
		arb_buffer_storage: extensions.contains(&"GL_ARB_buffer_storage"),
	}
}
