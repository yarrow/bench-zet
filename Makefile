all: comparison.txt self-comparison.txt
	@ echo -n
.PHONY: all

comparison.txt: compare.sh
	sh ./compare.sh

self-comparison.txt: self-compare.sh
	sh ./self-compare.sh

compare.sh self-compare.sh &: target/release/bench-zet
	cargo run --release

target/release/bench-zet: src/main.rs
	cargo build --release
