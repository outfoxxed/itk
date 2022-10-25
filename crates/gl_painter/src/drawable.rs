use std::error::Error;

use gl::types::GLenum;

use crate::{
	shader::{Shader, ShaderProgram, ShaderType},
	upload::VertexAttribute,
};

pub mod colored_triangle;
pub use colored_triangle::*;

pub trait Drawable {
	type DrawableData: bytemuck::Pod + VertexPassable;
	type Vertex: bytemuck::Pod + VertexPassable;

	const GL_TYPE: GLenum;
	const SHADER_SOURCE: ShaderSource;

	fn drawable_data(&self) -> Self::DrawableData;
	fn drawable_vertices(&self, vertices: &mut Vec<Self::Vertex>, indices: &mut Vec<u32>);
}

pub trait VertexPassable {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute];
}

pub struct ShaderSource {
	vertex_compat: &'static str,
	// TODO: check efficiency of SSBOs vs UBOs, and if its even possible to use them
	vertex_ssbo: &'static str,
	fragment: &'static str,
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

// TODO: 3d points
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct Point {
	pub x: f32,
	pub y: f32,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct Triangle(pub [Point; 3]);

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct Circle {
	origin: Point,
	radius: f32,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct ColoredCircle {
	circle: Circle,
	color: Color,
}

impl VertexPassable for Point {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute] = &[VertexAttribute::new::<f32>(2)];
}

impl VertexPassable for Color {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute] = &[VertexAttribute::new::<f32>(4)];
}

impl VertexPassable for Triangle {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute] = &[VertexAttribute::new::<f32>(3)];
}

impl VertexPassable for ColoredCircle {
	const VERTEX_ATTRIBUTES: &'static [VertexAttribute] = &[
		VertexAttribute::new::<f32>(2),
		VertexAttribute::new::<f32>(1),
		VertexAttribute::new::<f32>(4),
	];
}

/*
impl Drawable for ColoredCircle {
	type DrawableData = ColoredCircle;
	type Vertex = Point;

	const GL_TYPE: GLenum = gl::TRIANGLES;

	fn drawable_data(&self) -> Self::DrawableData {
		*self
	}

	fn drawable_vertices(&self, vertices: &mut Vec<Self::Vertex>, indices: &mut Vec<u32>) {
		vertices.extend_from_slice(&[
			Point {
				x: self.circle.origin.x - self.circle.radius,
				y: self.circle.origin.y - self.circle.radius,
			},
			Point {
				x: self.circle.origin.x - self.circle.radius,
				y: self.circle.origin.y + self.circle.radius,
			},
			Point {
				x: self.circle.origin.x + self.circle.radius,
				y: self.circle.origin.y - self.circle.radius,
			},
			Point {
				x: self.circle.origin.x + self.circle.radius,
				y: self.circle.origin.y + self.circle.radius,
			},
		]);
		indices.extend_from_slice(&[0, 1, 2, 3, 2, 1]);
	}
}
*/
