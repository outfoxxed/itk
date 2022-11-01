@match use_ssbo
	@case compat
		#version 120 core
	@case ssbo
		#version 430 core
@endmatch

@match use_ssbo
	@case compat
		layout(location = 0) varying vec2 v_pos;
		layout(location = 1) varying vec4 v_color;

		varying vec4 f_color;
	@case ssbo
		layout(location = 0) in vec2 v_pos;
		layout(location = 1) in uint s_index;

		layout(std430, binding = 0) buffer DrawableSSBO {
			vec4 s_color[];
		};

		out vec4 f_color;
@endmatch

void main() {
	@match use_ssbo
		@case compat
			f_color = v_color;
		@case ssbo
			f_color = s_color[s_index];
	@endmatch

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
