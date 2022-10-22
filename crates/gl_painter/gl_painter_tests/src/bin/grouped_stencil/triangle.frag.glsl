#version 330 core

uniform usampler2D stencil;

in VS_OUT {
	vec3 color;
	vec2 stencil_pos;
	flat uint stencil_id;
} var;

void main() {
	uint stencil_v = texture(stencil, var.stencil_pos).r;
	if (stencil_v == var.stencil_id) {
		gl_FragColor = vec4(var.color, 1.0);
	} else {
		gl_FragColor = vec4(var.color, 0.1);
	}
}
