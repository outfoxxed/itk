#version 430 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in uint s_index;

layout(std430, binding = 0) buffer DrawableSSBO {
	vec4 s_color[];
};

out vec4 f_color;

void main() {
	f_color = s_color[s_index];

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
