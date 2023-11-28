.PHONY: all clean opt deep-clean install run test

PREFIX=~/bin
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
BUILD_OUTPUT=sidejs.wasm phatjs.wasm

all: $(BUILD_OUTPUT)

%.wasm:
	cargo build --release --target wasm32-wasi --no-default-features
	cp $(BUILD_OUTPUT_DIR)/$@ $@

native:
	cargo build --release

install: native
	cp target/release/sidevm-quickjs $(PREFIX)/

run: all
	RUST_LOG=info sidevm-host $(BUILD_OUTPUT_SIDEJS)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
