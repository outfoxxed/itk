@match use_ssbo
	@case compat
		@define glsl_target 2.1
	@case ssbo
		@define glsl_target 4.3
@endmatch

@match glsl_target
	@case 2.1
		#version 120

		#define IN(loc) attribute
		#define OUT varying
	@case 4.3
		#version 430 core

		#define IN(loc) layout(location = loc) in
		#define OUT out
@endmatch

IN(0) vec2 v_pos;

@match use_ssbo
	@case compat
		IN(1) vec4 v_color;
	@case ssbo
		IN(1) uint s_index;

		layout(std430, binding = 0) buffer DrawableSSBO {
			vec4 s_color[];
		};
@endmatch

OUT vec4 f_color;

void main() {
	@match use_ssbo
		@case compat
			f_color = v_color;
		@case ssbo
			f_color = s_color[s_index];
	@endmatch

	gl_Position = vec4(v_pos, 0.0, 1.0);
}
