use gl::types::GLenum;

use super::{Color, Drawable, Point, ShaderSource, Triangle};

pub struct ColoredTriangle {
	pub triangle: Triangle,
	pub color: Color,
}

impl Drawable for ColoredTriangle {
	type DrawableData = Color;
	type Vertex = Point;

	const GL_TYPE: GLenum = gl::TRIANGLES;
	const SHADER_SOURCE: ShaderSource = ShaderSource {
		vertex_compat: include_str!("vertex_compat.glsl"),
		vertex_ssbo: include_str!("vertex_ssbo.glsl"),
		fragment: include_str!("fragment.glsl"),
	};

	fn drawable_data(&self) -> Self::DrawableData {
		self.color
	}

	fn drawable_vertices(&self, vertices: &mut Vec<Self::Vertex>, indices: &mut Vec<u32>) {
		vertices.extend_from_slice(&self.triangle.0);
		indices.extend_from_slice(&[0, 1, 2]);
	}
}
