@define test_inc a
@include include.glsl

test1

test2

@define test_inc b
@include include.glsl
@define test_inc c
@include include.glsl

test3

@define outer_match c
@match outer_match
	@case a
		@define defined_in_cold_case asd
		@match defined_in_cold_case
			@case asd
			@case fgh
		@endmatch
	@case b
		do_not_show
	@case c
		test4
@endmatch
