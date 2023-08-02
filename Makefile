.PHONY: all clean
all:
	cargo build --release --target wasm32-wasi

run: all
	sidevm-host target/wasm32-wasi/release/qjs.wasm

clean:
	cargo clean
	make clean -C qjs-sys
