.PHONY: all

all: example

example:
	# Dioxus CLI doesn't seem to allow for a custom config
	mv Dioxus.toml Dioxus.toml.temp || true
	cp Dioxus.prime-ministers.toml Dioxus.toml

	# Build release
	dx build --example prime_ministers --release

	# Restore config
	rm Dioxus.toml
	mv Dioxus.toml.temp Dioxus.toml || true
