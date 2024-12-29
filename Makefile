run:
	cargo run -- --cache cache

build:
	cargo build

watch.mac:
	fswatch -o ./src | xargs -I{} make build
