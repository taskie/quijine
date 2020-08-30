.PHONY: build

build:
	cargo build --release

PREFIX := $(HOME)/.local

.PHONY: install

install:
	$(MAKE) -C jj install

.PHONY: fmt

fmt:
	cargo +nightly fmt --all

.PHONY: fix

fix:
	cargo fix --allow-staged

.PHONY: doc

doc:
	cargo doc --open
