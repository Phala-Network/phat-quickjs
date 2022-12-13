.PHONY: all clean
all:
	cargo +nightly contract build --release --skip-linting
clean:
	cargo clean
	make clean -C qjs-sys
