TARGETS=wapojs

PREFIX=~/bin
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
BUILD_OUTPUT=$(addsuffix .wasm, $(TARGETS))
OPTIMIZED_OUTPUT=$(addsuffix -stripped.wasm, $(TARGETS))
WEB_BUILD_OUTPUT_DIR=target/wasm32-unknown-unknown/release
OPT?=0

.PHONY: all clean opt deep-clean install run test wasi rs

all: wasi

$(BUILD_OUTPUT): rs

wasi: $(BUILD_OUTPUT)

%.wasm:
	cargo build --release --target wasm32-wasi --no-default-features --features js-hash,wapo,js-http-listen,js-crypto,js-wasm
	cp $(BUILD_OUTPUT_DIR)/$@ $@

opt: all $(OPTIMIZED_OUTPUT)
	blake2 -b --length 32 *.wasm | tee hash.txt

%-stripped.wasm: %.wasm
	wasm-opt $< -Os -o $@
	wasm-tools strip $@ -o $@

native:
	cargo build --release --target x86_64-unknown-linux-musl --no-default-features --features mem-stats,native,js-hash,js-crypto,js-http-listen,js-wasm
	cp target/x86_64-unknown-linux-musl/release/wapojs ./

install: native
	$(foreach bin,$(TARGETS),cp target/release/$(bin) $(PREFIX)/;)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf $(WEB_BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm
	rm -rf wapojs

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
