all:
	bat --paging=never golden.b
	cargo run golden.b > out.ssa
	bat --paging=never out.ssa
	./qbe out.ssa > out.s
	# bat --paging=never out.s
	cc out.s

