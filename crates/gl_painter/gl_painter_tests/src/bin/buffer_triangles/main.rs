// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

//! Test to make sure buffers are working as expected

use gl_painter::{drawable::ColoredTriangle, upload, upload::Uploader};

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

						uploader.write(&ColoredTriangle {
							points: [
								[h - 0.1, v].into(),
								[h + 0.1, v].into(),
								[h, v + 0.1].into(),
							],
							color: [1.0, 1.0, 1.0, 1.0].into(),
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
