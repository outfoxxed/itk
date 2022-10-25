//! Test to make sure buffers are working as expected

use gl_painter::{
	drawable::{Color, ColoredTriangle, Point, Triangle},
	upload,
	upload::Uploader,
};

fn main() {
	gl_painter_tests::view_window(true, || {
		let mut uploader = unsafe { upload::ssbo::SsboUploader::new() };

		let mut anim_t = 0.0;

		// loop
		move || {
			{
				anim_t += 0.008;

				unsafe {
					uploader.prepare_write();
					uploader.clear();

					for i in 0..10 {
						if anim_t < i as f32 * 1.25 {
							continue
						}
						let h = (anim_t - (i as f32 * 0.15)) % 2.0 - 1.0;
						let v = -0.8 + i as f32 * 0.15;

						uploader.write(ColoredTriangle {
							triangle: Triangle([
								Point { x: h - 0.1, y: v },
								Point { x: h + 0.1, y: v },
								Point { x: h, y: v + 0.1 },
							]),
							color: Color {
								r: 1.0,
								g: 1.0,
								b: 1.0,
								a: 1.0,
							},
						});
					}

					uploader.begin_flush();
				}
			}

			unsafe {
				gl::ClearColor(0.2, 0.2, 0.2, 1.0);
				gl::Clear(gl::COLOR_BUFFER_BIT);

				uploader.upload();
			}
		}
	});
}
