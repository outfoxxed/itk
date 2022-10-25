#version 330 core

layout(location = 0) in vec2 v_pos;
layout(location = 1) in vec4 v_color;

out vec4 f_color;

void main() {
	f_color = v_color;

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
