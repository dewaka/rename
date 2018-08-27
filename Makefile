all: rename

rename:
	cargo build

clean:
	cargo clean

run:
	cargo run

.PHONY: clean run
