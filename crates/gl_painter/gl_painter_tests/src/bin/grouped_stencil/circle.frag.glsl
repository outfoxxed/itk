#version 330 core

in VS_OUT {
	vec2 pos;
	vec2 origin;
	float radius;
	vec3 color;
	flat uint stencil;
} var;

layout(location = 0) out vec4 draw_color;
layout(location = 1) out uint stencil;

float distanceSq(vec2 a, vec2 b) {
	vec2 v = a - b;
	return v.x * v.x + v.y * v.y;
}

void main() {
	float dSq = distanceSq(var.pos, var.origin);
	float rSq = var.radius * var.radius;
	if (dSq > rSq) discard;
	else {
		draw_color = vec4(var.color, 0.3);
		stencil = var.stencil;
	}
}
