all: rname

rname:
	cargo build

clean:
	cargo clean

run:
	cargo run

.PHONY: clean run
