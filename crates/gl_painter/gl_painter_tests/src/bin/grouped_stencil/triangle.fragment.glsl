#version 330 core

uniform usampler2D stencil;

in vec3 f_color;
in vec2 f_stencil_pos;
flat in uint f_stencil;

void main() {
	uint stencil_v = texture(stencil, f_stencil_pos).r;
	if (stencil_v == f_stencil) {
		gl_FragColor = vec4(f_color, 1.0);
	} else {
		gl_FragColor = vec4(f_color, 0.1);
	}
}
