.PHONY: build check small run size clean

BIN := target/x86_64-unknown-linux-gnu/release/termdemo
SMALL := target/x86_64-unknown-linux-gnu/release/termdemo-small

build:
	cargo build --release

check:
	cargo check --release

small: build
	objcopy --strip-section-headers $(BIN) $(SMALL)

run: build
	$(BIN)

size: small
	wc -c $(BIN)
	wc -c $(SMALL)
	size $(BIN)

clean:
	cargo clean
