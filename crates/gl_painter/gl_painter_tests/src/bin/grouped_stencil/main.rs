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

use std::ffi::CString;

use gl::types::GLenum;
use gl_painter::{
	drawable::{Drawable, ShaderSource},
	drawable_data,
	upload::{self, Uploader},
};
use rand::Rng;

fn main() {
	gl_painter_tests::view_window(true, || {
		let mut triangle_uploader = unsafe { upload::compat::CompatUploader::<Triangle>::new() };
		let mut circle_uploader = unsafe { upload::ssbo::SsboUploader::<Circle>::new() };

		struct StencilGroup {
			stencil: u16,
			color: [f32; 3],
		}

		let stencil_groups = (1..4)
			.into_iter()
			.map(|i| StencilGroup {
				stencil: i,
				color: [
					rand::thread_rng().gen_range(0.0..1.0),
					rand::thread_rng().gen_range(0.0..1.0),
					rand::thread_rng().gen_range(0.0..1.0),
				],
			})
			.collect::<Vec<_>>();

		let mut circles = stencil_groups
			.iter()
			.map(|group| MovingShape {
				shape: Circle {
					origin: [
						rand::thread_rng().gen_range(-0.5..0.5),
						rand::thread_rng().gen_range(-0.5..0.5),
					],
					radius: rand::thread_rng().gen_range(0.2..0.4),
					color: group.color,
					stencil: group.stencil,
				},
				movement: [
					rand::thread_rng().gen_range(-0.2..0.2),
					rand::thread_rng().gen_range(-0.2..0.2),
				],
				bounce: true,
				offscreen: false,
			})
			.collect::<Vec<_>>();

		let mut triangles = Vec::<MovingShape<Triangle>>::new();

		unsafe fn upload_circles(
			uploader: &mut upload::ssbo::SsboUploader<Circle>,
			circles: &[MovingShape<Circle>],
		) {
			uploader.prepare_write();
			uploader.clear();
			circles.iter().for_each(|c| uploader.write(&c.shape));
			uploader.begin_flush();
		}

		unsafe fn upload_triangles(
			uploader: &mut upload::compat::CompatUploader<Triangle>,
			triangles: &[MovingShape<Triangle>],
		) {
			uploader.prepare_write();
			uploader.clear();
			triangles.iter().for_each(|t| uploader.write(&t.shape));
			uploader.begin_flush();
		}

		let target_texture = unsafe {
			let mut texture = 0;
			gl::GenTextures(1, &mut texture);
			gl::BindTexture(gl::TEXTURE_2D, texture);

			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				gl::RGBA as i32,
				1000,
				1000,
				0,
				gl::RGBA,
				gl::UNSIGNED_BYTE,
				std::ptr::null(),
			);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

			texture
		};

		let stencil_texture = unsafe {
			let mut texture = 0;
			gl::GenTextures(1, &mut texture);
			gl::BindTexture(gl::TEXTURE_2D, texture);

			gl::TexImage2D(
				gl::TEXTURE_2D,
				0,
				gl::R16UI as i32,
				1000,
				1000,
				0,
				gl::RED_INTEGER,
				gl::UNSIGNED_SHORT,
				std::ptr::null(),
			);

			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
			gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

			texture
		};

		let framebuffer = unsafe {
			let mut fbo = 0;
			gl::GenFramebuffers(1, &mut fbo);
			gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

			gl::FramebufferTexture2D(
				gl::DRAW_FRAMEBUFFER,
				gl::COLOR_ATTACHMENT0,
				gl::TEXTURE_2D,
				target_texture,
				0,
			);
			gl::FramebufferTexture(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT1, stencil_texture, 0);

			fbo
		};

		unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) };

		let start_t = std::time::Instant::now();
		let mut last_delta = std::time::Duration::ZERO;
		let mut last_triangle = std::time::Instant::now();

		move || unsafe {
			gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer);

			gl::ClearColor(0.2, 0.2, 0.2, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);

			gl::Enable(gl::BLEND);
			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

			gl::DrawBuffers(2, &[gl::COLOR_ATTACHMENT0, gl::COLOR_ATTACHMENT1] as *const [u32]
				as *const GLenum);
			gl::ClearBufferuiv(gl::COLOR, 1, &0);
			circle_uploader.bind();
			circle_uploader.upload();
			gl::DrawBuffers(1, &[gl::COLOR_ATTACHMENT0] as *const [u32] as *const GLenum);

			gl::ActiveTexture(gl::TEXTURE1);
			gl::BindTexture(gl::TEXTURE_2D, stencil_texture);
			triangle_uploader.shader_program().bind();
			let stencil_str = CString::new("stencil").unwrap();
			gl::Uniform1i(
				gl::GetUniformLocation(
					triangle_uploader.shader_program().program_object,
					stencil_str.as_ptr(),
				),
				1,
			);
			triangle_uploader.bind();
			triangle_uploader.upload();

			gl::BindFramebuffer(gl::READ_FRAMEBUFFER, framebuffer);
			gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
			gl::BlitFramebuffer(
				0,
				0,
				1000,
				1000,
				0,
				0,
				1000,
				1000,
				gl::COLOR_BUFFER_BIT,
				gl::NEAREST,
			);

			upload_circles(&mut circle_uploader, &circles);
			upload_triangles(&mut triangle_uploader, &triangles);

			let now = std::time::Instant::now();
			let start_delta = now.duration_since(start_t);
			let current_delta = start_delta - last_delta;
			last_delta = start_delta;

			circles.iter_mut().for_each(|s| s.update(current_delta));
			triangles.iter_mut().for_each(|s| s.update(current_delta));
			triangles.retain(|triangle| !triangle.offscreen);

			let triangle_delta = now.duration_since(last_triangle);
			let triangle_count = triangle_delta.as_millis() / 20;

			if triangle_count > 0 {
				for _ in 0..triangle_count {
					let size = rand::thread_rng().gen_range(0.05..0.2);
					let group =
						&stencil_groups[rand::thread_rng().gen_range(0..stencil_groups.len())];
					triangles.push(MovingShape {
						shape: Triangle {
							position: [
								rand::thread_rng().gen_range(-1.0 + size..(1.0 - size * 2.0)),
								-1.0 - size,
							],
							size,
							color: group.color,
							stencil: group.stencil,
						},
						movement: [0.0, rand::thread_rng().gen_range(0.05..0.2)],
						bounce: false,
						offscreen: false,
					});
				}
				if triangle_count > 0 {
					last_triangle = now;
				}
			}
		}
	});
}

drawable_data!(CircleData {
	origin: Vec2,
	radius: f32,
	color: Vec3,
	stencil: u32,
});

drawable_data!(CircleVertex { position: Vec2 });

drawable_data!(TriangleData {
	color: Vec3,
	stencil: u32,
});

drawable_data!(TriangleVertex { position: Vec2 });

trait Shape {
	fn position(&mut self) -> &mut [f32; 2];
	fn shape(&self) -> [[f32; 2]; 2];
}

#[derive(Clone)]
struct Circle {
	origin: [f32; 2],
	radius: f32,
	color: [f32; 3],
	stencil: u16,
}

#[derive(Clone)]
struct Triangle {
	position: [f32; 2],
	size: f32,
	color: [f32; 3],
	stencil: u16,
}

impl Drawable for Circle {
	type Drawable = CircleData;
	type Vertex = CircleVertex;

	const GL_TYPE: GLenum = gl::TRIANGLES;
	const SHADER_SOURCE: ShaderSource = ShaderSource {
		vertex_compat: include_str!("circle.vertex_compat.glsl"),
		vertex_ssbo: include_str!("circle.vertex_ssbo.glsl"),
		fragment: include_str!("circle.fragment.glsl"),
	};

	fn drawable_data(&self) -> Self::Drawable {
		CircleData {
			origin: self.origin.into(),
			radius: self.radius,
			color: self.color.into(),
			stencil: self.stencil as u32,
		}
	}

	#[inline(always)]
	fn drawable_vertices<'s>(
		&'s self,
	) -> (
		impl IntoIterator<
			Item = Self::Vertex,
			IntoIter = impl ExactSizeIterator<Item = Self::Vertex> + 's,
		>,
		impl IntoIterator<Item = u32, IntoIter = impl ExactSizeIterator<Item = u32> + 's>,
	) {
		(
			[(-1, -1), (-1, 1), (1, -1), (1, 1)].map(|(x, y)| CircleVertex {
				position: [
					self.origin[0] + (self.radius * x as f32),
					self.origin[1] + (self.radius * y as f32),
				]
				.into(),
			}),
			[0, 1, 2, 1, 2, 3],
		)
	}
}

impl Drawable for Triangle {
	type Drawable = TriangleData;
	type Vertex = TriangleVertex;

	const GL_TYPE: GLenum = gl::TRIANGLES;
	const SHADER_SOURCE: ShaderSource = ShaderSource {
		vertex_compat: include_str!("triangle.vertex_compat.glsl"),
		vertex_ssbo: include_str!("triangle.vertex_ssbo.glsl"),
		fragment: include_str!("triangle.fragment.glsl"),
	};

	fn drawable_data(&self) -> Self::Drawable {
		TriangleData {
			color: self.color.into(),
			stencil: self.stencil as u32,
		}
	}

	#[inline(always)]
	fn drawable_vertices<'s>(
		&'s self,
	) -> (
		impl IntoIterator<
			Item = Self::Vertex,
			IntoIter = impl ExactSizeIterator<Item = Self::Vertex> + 's,
		>,
		impl IntoIterator<Item = u32, IntoIter = impl ExactSizeIterator<Item = u32> + 's>,
	) {
		(
			[
				[self.position[0], self.position[1] + self.size],
				[self.position[0] - self.size, self.position[1] - self.size],
				[self.position[0] + self.size, self.position[1] - self.size],
			]
			.into_iter()
			.map(|p| TriangleVertex { position: p.into() }),
			[0, 1, 2],
		)
	}
}

impl Shape for Circle {
	fn position(&mut self) -> &mut [f32; 2] {
		&mut self.origin
	}

	#[rustfmt::skip]
	fn shape(&self) -> [[f32; 2]; 2] {
		[
			[-self.radius, -self.radius],
			[self.radius, self.radius],
		]
	}
}

impl Shape for Triangle {
	fn position(&mut self) -> &mut [f32; 2] {
		&mut self.position
	}

	#[rustfmt::skip]
	fn shape(&self) -> [[f32; 2]; 2] {
		[
			[-self.size, -self.size],
			[self.size, self.size],
		]
	}
}

struct MovingShape<S: Shape> {
	shape: S,
	movement: [f32; 2],
	bounce: bool,
	offscreen: bool,
}

impl<S: Shape> MovingShape<S> {
	fn update(&mut self, delta: std::time::Duration) {
		let m = delta.as_secs_f32() * 10.0;

		let pos = *self.shape.position();
		let mut newpos = [pos[0] + self.movement[0] * m, pos[1] + self.movement[1] * m];

		let bounds = self.shape.shape();
		if self.bounce {
			for i in 0..2 {
				if newpos[i] + bounds[1][i] > 1.0 || newpos[i] + bounds[0][i] < -1.0 {
					self.movement[i] = -self.movement[i];
					newpos[i] = pos[i] + self.movement[i] * m;
				}
			}
		} else {
			for i in 0..2 {
				if newpos[i] + bounds[0][i] > 1.0 || newpos[i] + bounds[1][i] < -1.0 {
					self.offscreen = true;
				}
			}
		}

		*self.shape.position() = newpos;
	}
}
