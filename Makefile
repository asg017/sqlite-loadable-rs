
ifdef python
PYTHON=$(python)
else
PYTHON=python3
endif

test:
	cargo test
	cargo test --features=exec
	cargo test --features=static
	cargo build --examples --features=
	$(PYTHON) examples/test-examples.py
