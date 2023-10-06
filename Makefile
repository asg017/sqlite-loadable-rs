test:
	cargo test
	cargo test --features=exec
	cargo test --features=static
	cargo build --examples --features=
	python3 examples/test-examples.py
