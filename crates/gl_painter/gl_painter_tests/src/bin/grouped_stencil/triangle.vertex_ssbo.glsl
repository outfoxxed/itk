#version 430 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in uint s_index;

struct DrawableSSBO {
	vec4 color;
	uint stencil;
};

layout(std430, binding = 0) buffer SSBO {
	DrawableSSBO ssbo[];
};

out vec3 f_color;
out vec2 f_stencil_pos;
flat out uint f_stencil;

void main() {
	DrawableSSBO v_ssbo = ssbo[s_index];
	f_color = v_ssbo.color.rgb;
	f_stencil_pos = (v_pos + 1.0) / 2.0;
	f_stencil = v_ssbo.stencil;

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
