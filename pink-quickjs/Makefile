.PHONY: all clean
all:
	cargo contract build --release --max-memory-pages 64
clean:
	cargo clean
	make clean -C qjs-sys/qjs-sys
