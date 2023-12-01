.PHONY: all clean opt deep-clean
PREFIX=~/bin
BUILD_OUTOUT=target/wasm32-wasi/release/sidevm-quickjs.wasm

qjs.wasm: $(BUILD_OUTOUT)
	cp $(BUILD_OUTOUT) qjs.wasm

opt: qjs-opt.wasm

qjs-opt.wasm: qjs.wasm
	wasm-opt -Oz -o qjs-opt.wasm qjs.wasm
	wasm-strip qjs-opt.wasm

$(BUILD_OUTOUT):
	cargo build --release --target wasm32-wasi --no-default-features

native:
	cargo build --release

install: native
	cp target/release/sidevm-quickjs $(PREFIX)/

run: all
	RUST_LOG=info sidevm-host $(BUILD_OUTOUT)

clean:
	rm -rf $(BUILD_OUTOUT)
	rm -rf *.wasm

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test: qjs.wasm
	cd tests && yarn && yarn build && yarn bind && yarn test
