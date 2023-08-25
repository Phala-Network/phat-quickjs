.PHONY: all clean
all:
	cargo build --release --target wasm32-wasi --no-default-features

native:
	cargo build --release

run: all
	RUST_LOG=info sidevm-host target/wasm32-wasi/release/sidevm-quickjs.wasm

clean:
	cargo clean
	make clean -C qjs-sys
