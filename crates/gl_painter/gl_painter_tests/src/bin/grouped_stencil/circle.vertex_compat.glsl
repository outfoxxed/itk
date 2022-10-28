#version 330 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in vec2 v_origin;
layout(location = 2) in float v_radius;
layout(location = 3) in vec3 v_color;
layout(location = 4) in uint v_stencil;

out vec2 f_pos;
flat out vec2 f_origin;
flat out float f_radius;
flat out vec3 f_color;
flat out uint f_stencil;

void main() {
	f_pos = v_pos;
	f_origin = v_origin;
	f_radius = v_radius;
	f_color = v_color;
	f_stencil = v_stencil;

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
