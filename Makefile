.PHONY: docker-latest docker-fast

default: docker-fast

docker-latest: ## make `latest` docker image (full build in container)
	docker build -t bn254-rs:latest .

docker-fast: ## make `latest` docker image using host-built binary (faster)
	./build-fast.sh
