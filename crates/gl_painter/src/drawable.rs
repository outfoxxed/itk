// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use std::error::Error;

use gl::types::GLenum;

use crate::{
	shader::{Shader, ShaderProgram, ShaderType},
	upload::VertexAttribute,
};

pub mod colored_triangle;
pub use colored_triangle::*;

pub trait Drawable {
	type Drawable: DrawableData;
	type Vertex: DrawableData;

	const GL_TYPE: GLenum;
	const SHADER_SOURCE: ShaderSource;

	fn drawable_data(&self) -> Self::Drawable;
	fn drawable_vertices(&self, vertices: &mut Vec<Self::Vertex>, indices: &mut Vec<u32>);
}

pub trait VertexPassable {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute];
}

pub struct ShaderSource {
	pub vertex_compat: &'static str,
	pub vertex_ssbo: &'static str,
	pub fragment: &'static str,
}

impl ShaderSource {
	pub fn create_program(&self, use_ssbo: bool) -> ShaderProgram {
		let vertex_src = match use_ssbo {
			true => self.vertex_ssbo,
			false => self.vertex_compat,
		};

		(|| -> Result<ShaderProgram, Box<dyn Error>> {
			Ok(ShaderProgram::link(
				&Shader::compile(ShaderType::Vertex, vertex_src)?,
				&Shader::compile(ShaderType::Fragment, self.fragment)?,
			)?)
		})()
		.expect("Predefined shader could not be compiled")
	}
}

macro_rules! define_vec {
	(align($align:literal) $name:ident[$ty:ident; $count:literal]) => {
		::seq_macro::seq!(N in 0..$count {
			::paste::paste! {
				#[derive(Clone, Debug)]
				pub struct $name(#($ty,)*);

				#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
				#[repr(C, packed)]
				pub struct [<$name Packed>](#($ty,)*);

				#[derive(Copy, Clone, bytemuck::AnyBitPattern)]
				#[repr(C, align($align))]
				pub struct [<$name Aligned>](#($ty,)*);

				impl From<[$ty; $count]> for $name {
					fn from(v: [$ty; $count]) -> Self {
						Self(#(v[N],)*)
					}
				}

				impl From<$name> for [<$name Packed>] {
					fn from(r: $name) -> Self {
						Self(#(r.N,)*)
					}
				}

				impl From<$name> for [<$name Aligned>] {
					fn from(r: $name) -> Self {
						Self(#(r.N,)*)
					}
				}
			}
		});
	};
}

define_vec!(align(8) Vec2[f32; 2]);
define_vec!(align(16) Vec3[f32; 3]);
define_vec!(align(16) Vec4[f32; 4]);

pub trait DrawableData {
	type Ssbo: bytemuck::AnyBitPattern;
	type Compat: bytemuck::Pod + VertexPassable + std::fmt::Debug;

	fn into_ssbo(self) -> Self::Ssbo;
	fn into_compat(self) -> Self::Compat;
}

#[macro_export]
macro_rules! drawable_data {
	($name:ident { $($field:ident: $type:ident,)* }) => {
		::paste::paste! {
			#[derive(Clone)]
			pub struct $name {
				$($field: $crate::drawable::drawable_data!(|vr| $type),)*
			}

			#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
			#[repr(C, packed)]
			pub struct [<$name Packed>] {
				$($field: $crate::drawable::drawable_data!(|vp| $type),)*
			}

			#[derive(Copy, Clone, bytemuck::AnyBitPattern)]
			#[repr(C)]
			pub struct [<$name Aligned>] {
				$($field: $crate::drawable::drawable_data!(|va| $type),)*
			}

			impl $crate::drawable::VertexPassable for [<$name Packed>] {
				const VERTEX_ATTRIBUTES: &'static [$crate::upload::attribute::VertexAttribute] = &[
					$($crate::drawable::drawable_data!(|atr| $type),)*
				];
			}

			impl $crate::drawable::DrawableData for $name {
				type Ssbo = [<$name Aligned>];
				type Compat = [<$name Packed>];

				#[inline]
				fn into_ssbo(self) -> Self::Ssbo {
					self.into()
				}

				#[inline]
				fn into_compat(self) -> Self::Compat {
					self.into()
				}
			}

			impl From<$name> for [<$name Packed>] {
				#[inline]
				fn from(r: $name) -> Self {
					Self {
						$($field: $crate::drawable::drawable_data!(|i| r.$field: $type),)*
					}
				}
			}

			impl From<$name> for [<$name Aligned>] {
				#[inline]
				fn from(r: $name) -> Self {
					Self {
						$($field: $crate::drawable::drawable_data!(|i| r.$field: $type),)*
					}
				}
			}
		}
	};

	(|vr| Vec2) => { $crate::drawable::Vec2 };
	(|vr| Vec3) => { $crate::drawable::Vec3 };
	(|vr| Vec4) => { $crate::drawable::Vec4 };
	(|vr| $type:ident) => { $type };

	(|vp| Vec2) => { $crate::drawable::Vec2Packed };
	(|vp| Vec3) => { $crate::drawable::Vec3Packed };
	(|vp| Vec4) => { $crate::drawable::Vec4Packed };
	(|vp| $type:ident) => { $type };

	(|va| Vec2) => { $crate::drawable::Vec2Aligned };
	(|va| Vec3) => { $crate::drawable::Vec3Aligned };
	(|va| Vec4) => { $crate::drawable::Vec4Aligned };
	(|va| $type:ident) => { $type };

	(|atr| Vec2) => { $crate::upload::attribute::VertexAttribute::new::<f32>(2) };
	(|atr| Vec3) => { $crate::upload::attribute::VertexAttribute::new::<f32>(3) };
	(|atr| Vec4) => { $crate::upload::attribute::VertexAttribute::new::<f32>(4) };
	(|atr| $type:ident) => { $crate::upload::attribute::VertexAttribute::new::<$type>(1) };

	(|i| $v:ident.$name:ident: Vec2) => { $v.$name.into() };
	(|i| $v:ident.$name:ident: Vec3) => { $v.$name.into() };
	(|i| $v:ident.$name:ident: Vec4) => { $v.$name.into() };
	(|i| $v:ident.$name:ident: $type:ident) => { $v.$name };
}

pub use drawable_data;
