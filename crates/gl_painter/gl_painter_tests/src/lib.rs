// Copyright (C) 2022 the ITK authors
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/./

use glfw::{Context, OpenGlProfileHint, WindowHint};

pub mod debug;

pub fn view_window<I: FnOnce() -> L, L: FnMut()>(vsync: bool, test: I) {
	let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
	glfw.window_hint(WindowHint::ContextVersion(3, 3));
	glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
	glfw.window_hint(WindowHint::OpenGlDebugContext(true));

	let (mut window, events) =
		glfw.create_window(1000, 1000, "test", glfw::WindowMode::Windowed).unwrap();

	window.make_current();

	if !vsync {
		glfw.set_swap_interval(glfw::SwapInterval::None);
	}

	window.set_size_polling(true);

	gl::load_with(|p| window.get_proc_address(p));

	env_logger::init();
	debug::setup_gl_debug();

	let mut test_loop = test();
	while !window.should_close() {
		test_loop();

		window.swap_buffers();
		glfw.poll_events();
		for (_, event) in glfw::flush_messages(&events) {
			match event {
				glfw::WindowEvent::Size(width, height) => unsafe {
					gl::Viewport(0, 0, width, height);
				},
				_ => {},
			}
		}
	}
}
