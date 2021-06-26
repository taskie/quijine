.PHONY: build

build:
	cargo build --all --release

PREFIX := $(HOME)/.local

.PHONY: install

install:
	$(MAKE) -C quijine_jj install

.PHONY: dev

dev:
	cargo build --all

.PHONY: test

test:
	cargo test --all

.PHONY: fmt

fmt:
	cargo +nightly fmt --all

.PHONY: fix

fix:
	cargo +nightly fix --all --allow-staged

.PHONY: vet

lint:
	cargo +nightly clippy --all --all-features

.PHONY: doc

doc:
	cargo doc --open

.PHONY: pre-commit

pre-commit:
	$(MAKE) fix
	$(MAKE) fmt
	git diff --exit-code
	$(MAKE) lint
	$(MAKE) test

.PHONY: setup-dev

setup-dev:
	echo 'exec make pre-commit 1>&2' >.git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit
