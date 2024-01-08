TARGETS=sidejs phatjs

PREFIX=~/bin
BUILD_OUTPUT_DIR=target/wasm32-wasi/release
BUILD_OUTPUT=$(addsuffix .wasm, $(TARGETS))
OPTIMIZED_OUTPUT=$(addsuffix -stripped.wasm, $(TARGETS))
WEB_BUILD_OUTPUT_DIR=target/wasm32-unknown-unknown/release
OPT?=0

.PHONY: all clean opt deep-clean install run test web phatjs-web.wasm wasi

all: wasi web native
wasi: $(BUILD_OUTPUT)
web: phatjs-web.wasm
	wasm-bindgen phatjs-web.wasm --out-dir web/dist --typescript --target web --out-name phatjs
ifeq ($(OPT),1)
	wasm-opt web/dist/phatjs_bg.wasm -Os -o web/dist/phatjs_bg.wasm
	wasm-tools strip web/dist/phatjs_bg.wasm -o web/dist/phatjs_bg.wasm
endif
	tar czvf phatjs-web.tar.gz web/

%.wasm:
	cargo build --release --target wasm32-wasi --no-default-features --features js-hash,sidevm
	cp $(BUILD_OUTPUT_DIR)/$@ $@

phatjs-web.wasm:
	cargo build --bin phatjs --release --target wasm32-unknown-unknown --no-default-features --features js-hash,web,mem-stats
	cp $(WEB_BUILD_OUTPUT_DIR)/phatjs.wasm $@

opt: all $(OPTIMIZED_OUTPUT)
	blake2 -b --length 32 *.wasm phatjs-x86_64-unknown-linux-musl | tee hash.txt

%-stripped.wasm: %.wasm
	wasm-opt $< -Os -o $@
	wasm-tools strip $@ -o $@

native:
	cargo build --release --target x86_64-unknown-linux-musl --features mem-stats
	cp target/x86_64-unknown-linux-musl/release/phatjs phatjs-x86_64-unknown-linux-musl

install: native
	$(foreach bin,$(TARGETS),cp target/release/$(bin) $(PREFIX)/;)

clean:
	rm -rf $(BUILD_OUTPUT_DIR)/*.wasm
	rm -rf $(WEB_BUILD_OUTPUT_DIR)/*.wasm
	rm -rf *.wasm
	rm -rf phatjs-*

deep-clean: clean
	cargo clean
	make clean -C qjs-sys/qjs-sys

test:
	cd tests && yarn && yarn build && yarn bind && yarn test
