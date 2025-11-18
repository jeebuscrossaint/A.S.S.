.PHONY: build release install

build:
	cargo build --release
	@mkdir -p bin
	@cp target/release/ass bin/ass
	@echo "✓ Binary copied to bin/ass"

release: build
	@echo "✓ Release build complete"

install:
	cargo install --path .

clean:
	cargo clean
	rm -rf bin/

run:
	cargo run --release
