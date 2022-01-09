run:
	cargo run --release --offline -- transactions.csv > accounts.csv

build: 
	cargo build --release --offline

build_no_color: 
	cargo build --release --offline --features="no_color"

test:
	cargo test --offline

clippy: 
	cargo clippy --offline

clean: 
	rm -r target; \
	rm Cargo.lock
