// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use gl::types::GLenum;

pub struct VertexAttribute {
	pub ty: GLenum,
	pub count: usize,
	pub ty_size: usize,
	pub is_integer: bool,
}

impl VertexAttribute {
	pub const fn new<T: GLtype>(count: usize) -> Self {
		VertexAttribute {
			ty: T::GL_TYPE,
			count,
			ty_size: std::mem::size_of::<T>(),
			is_integer: T::IS_INTEGER,
		}
	}
}

pub trait GLtype: Sized {
	const GL_TYPE: GLenum;
	const IS_INTEGER: bool;
}

macro_rules! gl_types {
	($($type:ident($gltype:expr, int: $int:literal);)*) => {
		$(
			impl GLtype for $type {
				const GL_TYPE: GLenum = $gltype;
				const IS_INTEGER: bool = $int;
			}
		)*
	}
}

gl_types! {
	f64(gl::DOUBLE, int: false);
	f32(gl::FLOAT, int: false);

	u32(gl::UNSIGNED_INT, int: true);
	u16(gl::UNSIGNED_SHORT, int: true);
	u8(gl::UNSIGNED_BYTE, int: true);

	i32(gl::INT, int: true);
	i16(gl::SHORT, int: true);
	i8(gl::BYTE, int: true);
}
