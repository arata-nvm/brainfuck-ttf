BASE_FONT=OpenSans-Regular.ttf
TARGET_FONT=brainfuck.ttf
WASM=shaper.wasm

$(TARGET_FONT): $(WASM) $(BASE_FONT)
	./bin/otfsurgeon -i $(BASE_FONT) add -o $(TARGET_FONT) Wasm < $(WASM)

$(WASM): crates/shaper/src/lib.rs
	cargo build --target=wasm32-wasip1 --release
	cp target/wasm32-wasip1/release/shaper.wasm $(WASM)

.PHONY: clean
clean:
	rm $(WASM)
