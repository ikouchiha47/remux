run:
	cargo run -- --cache cache

watch.mac:
	fswatch -o ./src | xargs -I{} make run
