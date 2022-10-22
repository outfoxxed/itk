#version 330 core
layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 origin;
layout(location = 2) in float radius;
layout(location = 3) in vec3 color;
layout(location = 4) in uint stencil;

out VS_OUT {
	vec2 pos;
	vec2 origin;
	float radius;
	vec3 color;
	flat uint stencil;
} var;

void main() {
	var.pos = pos;
	var.origin = origin;
	var.radius = radius;
	var.color = color;
	var.stencil = stencil;

	gl_Position = vec4(pos, 0.0, 1.0);
}
