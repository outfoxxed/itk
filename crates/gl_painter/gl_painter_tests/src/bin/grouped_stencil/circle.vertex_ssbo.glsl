#version 430 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in uint s_index;

struct DrawableSSBO {
	vec2 origin;
	float radius;
	vec4 color;
	uint stencil;
};

layout(std430, binding = 0) buffer SSBO {
	DrawableSSBO ssbo[];
};

out vec2 f_pos;
flat out vec2 f_origin;
flat out float f_radius;
flat out vec3 f_color;
flat out uint f_stencil;

void main() {
	DrawableSSBO v_ssbo = ssbo[s_index];
	f_pos = v_pos;
	f_origin = v_ssbo.origin;
	f_radius = v_ssbo.radius;
	f_color = v_ssbo.color.rgb;
	f_stencil = v_ssbo.stencil;

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
