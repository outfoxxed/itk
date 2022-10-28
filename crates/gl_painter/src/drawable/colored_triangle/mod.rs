// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use gl::types::GLenum;

use super::{drawable_data, Drawable, ShaderSource, Vec2, Vec4};

pub struct ColoredTriangle {
	pub points: [Vec2; 3],
	pub color: Vec4,
}

impl Drawable for ColoredTriangle {
	type Drawable = ColoredTriangleData;
	type Vertex = ColoredTriangleVertex;

	const GL_TYPE: GLenum = gl::TRIANGLES;
	const SHADER_SOURCE: ShaderSource = ShaderSource {
		vertex_compat: include_str!("vertex_compat.glsl"),
		vertex_ssbo: include_str!("vertex_ssbo.glsl"),
		fragment: include_str!("fragment.glsl"),
	};

	fn drawable_data(&self) -> Self::Drawable {
		ColoredTriangleData {
			color: self.color.clone(),
		}
	}

	fn drawable_vertices(&self, vertices: &mut Vec<Self::Vertex>, indices: &mut Vec<u32>) {
		vertices.extend(self.points.iter().map(|p| ColoredTriangleVertex { pos: p.clone() }));
		indices.extend_from_slice(&[0, 1, 2]);
	}
}

drawable_data!(ColoredTriangleData { color: Vec4 });

drawable_data!(ColoredTriangleVertex { pos: Vec2 });
