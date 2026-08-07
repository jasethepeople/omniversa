.PHONY: all build test bench clean docker deploy fmt lint docs

all: build test
build:
	cargo build --release
test:
	cargo test --workspace
bench:
	cargo bench
clean:
	cargo clean
	rm -rf target/
docker:
	docker-compose -f infra/docker/docker-compose.yml up --build
deploy:
	./scripts/deploy.sh production
fmt:
	cargo fmt --all
lint:
	cargo clippy --all-targets --all-features -- -D warnings
docs:
	cargo doc --no-deps --open
