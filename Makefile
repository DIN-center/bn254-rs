.PHONY: docker-latest

default: docker-latest

docker-latest: ## make `latest` docker image
	docker build -t bn254-rs:latest .
