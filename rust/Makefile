#####################

SRCS = step0_repl.rs step1_read_print.rs step2_eval.rs \
		step3_env.rs step4_if_fn_do.rs step5_tco.rs \
		step6_file.rs
#       step7_quote.rs \
#       step8_macros.rs step9_try.rs stepA_mal.rs
BINS = $(SRCS:%.rs=target/release/%)

#####################

all: $(BINS)

run:
	cargo run --bin main

readline:
	cargo build --features "readline"

release:
	cargo build --release
	./target/release/main


dist: mal

mal: target/release/stepA_mal
	cp $< $@

# TODO: would be nice to build just the step requested
$(BINS): target/release/%: src/bin/%.rs $(wildcard src/*.rs)
	cargo build --release

clean:
	cargo clean
	rm mal

.PHONY: mal run
