SHELL := /bin/sh

CARGO ?= cargo
NPM ?= npm
PREFIX ?= $(HOME)/.local
BINARY := now

.PHONY: help build release-build test fmt fmt-check lint npm-pack check install install-local clean

help:
	@printf '%s\n' \
		'Targets:' \
		'  make build          Build debug binary' \
		'  make release-build  Build release binary with Cargo.lock' \
		'  make test           Run Rust and npm tests' \
		'  make fmt            Format Rust code' \
		'  make fmt-check      Check Rust formatting' \
		'  make lint           Run clippy with warnings as errors' \
		'  make npm-pack       Dry-run npm package contents' \
		'  make check          Run formatting, lint, tests, and npm pack dry-run' \
		'  make install        Install release binary into $(HOME)/.local/bin' \
		'  make install-local  Install release binary into $(PREFIX)/bin' \
		'  make clean          Remove build artifacts'

build:
	$(CARGO) build

release-build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test
	$(NPM) test

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

npm-pack:
	$(NPM) pack --dry-run

check: fmt-check lint test npm-pack

install: install-local

install-local: release-build
	mkdir -p "$(PREFIX)/bin"
	cp "target/release/$(BINARY)" "$(PREFIX)/bin/$(BINARY)"
	chmod 755 "$(PREFIX)/bin/$(BINARY)"
	@printf 'Installed %s\n' "$(PREFIX)/bin/$(BINARY)"

clean:
	$(CARGO) clean
	rm -rf npm/*-bin .npm
