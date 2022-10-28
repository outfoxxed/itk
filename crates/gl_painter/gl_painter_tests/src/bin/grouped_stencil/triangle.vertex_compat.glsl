#version 330 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in vec3 v_color;
layout(location = 2) in uint v_stencil;

out vec3 f_color;
out vec2 f_stencil_pos;
flat out uint f_stencil;

void main() {
	f_color = v_color;
	f_stencil_pos = (v_pos + 1.0) / 2.0;
	f_stencil = v_stencil;

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
