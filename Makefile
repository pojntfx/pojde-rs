all: build

build:
	cargo build

release:
	cargo build --release
	
install: release
	cargo install --path .

dev:
	cargo watch
	
clean:
	cargo clean

depend:
	cargo fetch