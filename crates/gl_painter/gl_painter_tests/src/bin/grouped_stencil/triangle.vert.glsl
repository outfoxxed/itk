#version 330 core
layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 color;
layout(location = 2) in uint stencil;

out VS_OUT {
	vec3 color;
	vec2 stencil_pos;
	flat uint stencil_id;
} var;

void main() {
	var.color = color;
	var.stencil_pos = (pos + 1.0) / 2.0;
	var.stencil_id = stencil;

	gl_Position = vec4(pos, 0.0, 1.0);
}
