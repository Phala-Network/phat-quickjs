TARGETS=sidejs phatjs

PREFIX=~/bin
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
BUILD_OUTPUT=$(addsuffix .wasm, $(TARGETS))
OPTIMIZED_OUTPUT=$(addsuffix -stripped.wasm, $(TARGETS))

.PHONY: all clean opt deep-clean install run test

all: $(BUILD_OUTPUT)

%.wasm:
	cargo build --release --target wasm32-wasi --no-default-features --features js-hash
	cp $(BUILD_OUTPUT_DIR)/$@ $@

opt: $(OPTIMIZED_OUTPUT)

%-stripped.wasm: %.wasm
	wasm-opt $< -Os -o $@
	wasm-tools strip $@ -o $@

native:
	cargo build --release

install: native
	$(foreach bin,$(TARGETS),cp target/release/$(bin) $(PREFIX)/;)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
