#version 330 core

in vec2 f_pos;
flat in vec2 f_origin;
flat in float f_radius;
flat in vec3 f_color;
flat in uint f_stencil;

layout(location = 0) out vec4 draw_color;
layout(location = 1) out uint stencil;

void main() {
	vec2 d = f_pos - f_origin;
	float dSq = d.x * d.x + d.y * d.y;
	float rSq = f_radius * f_radius;

	if (dSq > rSq) discard;
	else {
		draw_color = vec4(f_color, 0.3);
		stencil = f_stencil;
	}
}
