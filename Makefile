.PHONY: all clean
all:
	cargo +nightly contract build --release
clean:
	cargo clean
	make clean -C qjs-sys
