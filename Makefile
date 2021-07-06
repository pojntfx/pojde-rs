all: build

build:
	# Create binary
	mkdir -p /tmp/out
	cargo build --release

	# Stage binaries
	mkdir -p out
	cp target/release/pojdectl-rs out/pojdectl-rs

release: build
	# Stage binaries
	mkdir -p out/release
	cp out/pojdectl-rs out/release/pojdectl-rs.linux-$$(uname -m)

install: release
	sudo install out/release/pojdectl-rs.linux-$$(uname -m) /usr/local/bin/pojdectl-rs

dev:
	cargo watch
	
clean:
	cargo clean

depend:
	# Install development dependencies
	cargo install cargo-watch

	# Install dependencies
	cargo build