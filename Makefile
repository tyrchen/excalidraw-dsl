build:
	@cargo build --all-features

# CLI commands
convert:
	@cargo run -- convert $(FILE)

validate:
	@cargo run -- validate $(FILE)

watch:
	@cargo run -- watch $(FILE)

server:
	@cargo run --features server -- server --port 3002

# Old compatibility targets
build-server:
	@cargo build --features server

run-server:
	@cargo run --features server -- server --port 3002

run-ui:
	@cd ui && yarn dev

run-full:
	@echo "Starting EDSL server and UI..."
	@cargo run --features server -- server --port 3002 & \
	cd ui && yarn dev & \
	wait

# Example conversions
examples:
	@echo "Converting example files..."
	@cargo run -- convert examples/simple.edsl -o examples/simple.json
	@cargo run -- convert examples/complex-architecture.edsl -o examples/complex-architecture.json
	@cargo run -- convert examples/decision-tree.edsl -o examples/decision-tree.json
	@cargo run -- convert examples/container-groups.edsl -o examples/container-groups.json

test:
	@cargo nextest run --all-features

test-cli:
	@cargo test --bin edsl

install:
	@cargo install --path . --features server

release:
	@cargo release tag --execute
	@git cliff -o CHANGELOG.md
	@git commit -a -n -m "Update CHANGELOG.md" || true
	@git push origin master
	@cargo release push --execute

update-submodule:
	@git submodule update --init --recursive --remote

.PHONY: build convert validate watch server build-server run-server run-ui run-full examples test test-cli install release update-submodule
