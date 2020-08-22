NAME := jj
BIN := ../target/release/$(NAME)

.PHONY: build

build:
	cargo build --release

PREFIX := $(HOME)/.local

.PHONY: install

install: build
	install -m755 $(BIN) $(PREFIX)/bin/$(NAME)
