test:
	cargo test -- --test-threads=1

test-lib:
	cargo test --lib -- --test-threads=1

test-bin:
	cargo test --bin tickets -- --test-threads=1

test-some:
	cargo test $(CARGOTEST) -- --test-threads=1
