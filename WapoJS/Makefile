TARGETS=wapojs

DEBUG?=0
PREFIX=~/bin

ifeq ($(DEBUG), 1)
BUILD_OUTPUT_DIR=target/wasm32-wasi/debug
NATIVE_OUTPUT_DIR=target/x86_64-unknown-linux-musl/debug
CARGO_PROFILE=
else
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
NATIVE_OUTPUT_DIR=target/x86_64-unknown-linux-musl/release
CARGO_PROFILE=--release
endif

BUILD_OUTPUT=$(addsuffix .wasm, $(TARGETS))
OPTIMIZED_OUTPUT=$(addsuffix -stripped.wasm, $(TARGETS))
OPT?=0
COMMON_FEATURES=mem-stats,js-hash,js-crypto,js-wasm,js-websocket,js-crypto,env-nodejs,js-https-listen,isolate


.PHONY: all clean opt deep-clean install run test wasi rs runner

all: wasi native runner

$(BUILD_OUTPUT): rs

runner: wapojs-run

wapojs-run: runner/target/release/wapojs-run
	cp $< $@

runner/target/release/wapojs-run:
	cd runner && cargo build --release

wasi: $(BUILD_OUTPUT)

%.wasm:
	cargo build $(CARGO_PROFILE) --target wasm32-wasi --no-default-features --features wapo,js-http-listen,$(COMMON_FEATURES)
	cp $(BUILD_OUTPUT_DIR)/$@ $@

opt: all $(OPTIMIZED_OUTPUT)
	blake2 -b --length 32 *.wasm | tee hash.txt

%-stripped.wasm: %.wasm
	wasm-opt $< -Os -o $@
	wasm-tools strip $@ -o $@

native:
	cargo build $(CARGO_PROFILE) --target x86_64-unknown-linux-musl --no-default-features --features native,$(COMMON_FEATURES)
	cp $(NATIVE_OUTPUT_DIR)/wapojs ./

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm
	rm -rf wapojs wapojs-run

install: native runner
	cp ./wapojs ~/.cargo/bin/
	cp ./wapojs-run ~/.cargo/bin/

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
