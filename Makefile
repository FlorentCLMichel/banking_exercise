run:
	cargo run --release --offline -- transactions.csv > accounts.csv

build: 
	cargo build --release --offline

test:
	cargo test --offline

clippy: 
	cargo clippy --offline

clean: 
	rm -r target; \
	rm Cargo.lock
