.PHONY: all test lint build clean example fmt

# Rust parameters
CARGO = cargo

all: test build

build:
	$(CARGO) build --release

test:
	$(CARGO) test --verbose

lint:
	$(CARGO) clippy -- -D warnings

fmt:
	$(CARGO) fmt

clean:
	$(CARGO) clean

example:
	@echo "Running example..."
	@$(CARGO) run --example simple_transfer

check:
	$(CARGO) check

doc:
	$(CARGO) doc --open
