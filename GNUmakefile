.PHONY: build

build:
	cargo build --all --release

PREFIX := $(HOME)/.local

.PHONY: install

install:
	$(MAKE) -C jj install

.PHONY: test

test:
	cargo test --all

.PHONY: fmt

fmt:
	cargo +nightly fmt --all

.PHONY: fix

fix:
	cargo fix --all --allow-staged

.PHONY: vet

vet:
	cargo clippy --all --all-features

.PHONY: doc

doc:
	cargo doc --open
