build:
	cargo build --release

install:
	cp target/release/nvimx /usr/local/bin/nvimx
