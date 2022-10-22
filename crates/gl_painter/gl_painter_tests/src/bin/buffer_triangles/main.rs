//! Test to make sure buffers are working as expected

use std::mem::MaybeUninit;

use gl::types::GLfloat;
use gl_painter::{
	shader::{Shader, ShaderProgram, ShaderType},
	upload,
};

fn main() {
	gl_painter_tests::view_window(true, || {
		let shader = ShaderProgram::link(
			&Shader::compile(ShaderType::Vertex, include_str!("shader.vert.glsl")).unwrap(),
			&Shader::compile(ShaderType::Fragment, include_str!("shader.frag.glsl")).unwrap(),
		)
		.unwrap();
		shader.bind();

		#[rustfmt::skip]
		let mut uploader = unsafe {
			upload::Uploader::new(gl::TRIANGLES, vec![
				upload::VertexAttribute::new::<f32>(3)
			])
		};

		let mut anim_t = 0.0;

		// loop
		move || {
			unsafe {
				gl::ClearColor(0.2, 0.2, 0.2, 1.0);
				gl::Clear(gl::COLOR_BUFFER_BIT);

				uploader.upload();
			}

			{
				anim_t += 0.008;

				unsafe {
					uploader.prepare_write();

					for i in 0..10 {
						if anim_t < i as f32 * 1.25 {
							continue
						}
						let h = (anim_t - (i as f32 * 0.15)) % 2.0 - 1.0;
						let v = -0.8 + i as f32 * 0.15;

						let (mut vbuf, mut ibuf) = uploader.write();

						vbuf.write(
							i * 3,
							std::mem::transmute::<&[[f32; 3]], &[MaybeUninit<[f32; 3]>]>(&[
								[h - 0.1, v, 0.0],
								[h + 0.1, v, 0.0],
								[h, v + 0.1, 0.0],
							]),
						);
						ibuf.write(
							i * 3,
							std::mem::transmute::<&[u32], &[MaybeUninit<u32>]>(&[
								(i * 3 + 0) as u32,
								(i * 3 + 1) as u32,
								(i * 3 + 2) as u32,
							]),
						);
					}

					uploader.begin_flush();
				}
			}
		}
	});
}
