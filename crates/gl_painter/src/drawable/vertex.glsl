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
