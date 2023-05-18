.PHONY: all clean
all:
	cargo contract build --release
clean:
	cargo clean
	make clean -C qjs-sys
