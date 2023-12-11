TARGETS=sidejs phatjs

PREFIX=~/bin
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
BUILD_OUTPUT=$(addsuffix .wasm, $(TARGETS))
OPTIMIZED_OUTPUT=$(addsuffix -stripped.wasm, $(TARGETS))
WEB_BUILD_OUTPUT_DIR=target/wasm32-unknown-unknown/release

.PHONY: all clean opt deep-clean install run test web phatjs-web.wasm wasi

all: wasi web
wasi: $(BUILD_OUTPUT)
web: phatjs-web.wasm
	-wasm-bindgen phatjs-web.wasm  --out-dir l --typescript --target web --out-name index

%.wasm:
	cargo build --release --target wasm32-wasi --no-default-features --features js-hash,sidevm
	cp $(BUILD_OUTPUT_DIR)/$@ $@

phatjs-web.wasm:
	cargo build --bin phatjs --release --target wasm32-unknown-unknown --no-default-features --features js-hash,web
	cp $(WEB_BUILD_OUTPUT_DIR)/phatjs.wasm $@

opt: all $(OPTIMIZED_OUTPUT)

%-stripped.wasm: %.wasm
	wasm-opt $< -Os -o $@
	wasm-tools strip $@ -o $@

native:
	cargo build --release

install: native
	$(foreach bin,$(TARGETS),cp target/release/$(bin) $(PREFIX)/;)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf $(WEB_BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
