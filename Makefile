.PHONY: docker-latest docker-fast

default: build

docker-latest: ## make `latest` docker image (full build in container)
	docker build -t bn254-rs:latest .

docker-fast: ## make `latest` docker image using host-built binary (faster)
	./build-fast.sh


clean: ## clean project
	cargo clean

build: ## build project
	cargo build --bin txtx-bn254-signer

serve: ## start server with random salt
	@SALT=$$(openssl rand -hex 32) && \
	echo "Starting server with salt: $$SALT" && \
	cargo run --bin txtx-bn254-signer -- --salt "$$SALT"

test-sign: ## test sign endpoint with curl
	@EOA="0x70997970C51812dc3A010C7d01b50e0d17dc79C8" && \
	MSG_HASH="1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeffedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321" && \
	curl -s -X POST http://localhost:3000/sign \
	  -H "Content-Type: application/json" \
	  -d "{ \
	    \"eoa_address\": \"$$EOA\", \
	    \"message\": \"$$MSG_HASH\" \
	  }" | jq .

test-validate: ## validate signature cryptographically with Python (full diagnostics)
	@bash -c "source .venv/bin/activate && python test.py"

test-quick: ## quick validation - only check curve points and pairing
	@bash -c "source .venv/bin/activate && python test.py --quick"




