run:
	cargo run 

build:
	cargo build

watch.mac:
	fswatch -o ./src | xargs -I{} make build
