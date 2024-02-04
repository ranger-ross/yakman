help: ## Shows help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

redis: ## Starts Redis with Docker
	docker run -it --rm -p 6379:6379 -v $(shell pwd)/testing-directory/redis:/data redis

mock-auth: ## Starts a mock OAuth server with Docker
	docker compose -f mock-openid-connect-server/docker-compose.yaml up

build-backend: ## Builds the backend dockerfile with tag local/yakman-backend
	docker build . -f backend.Dockerfile -t local/yakman-backend

build-frontend: ## Builds the frontend dockerfile with tag local/yakman-frontend
	docker build . -f frontend.Dockerfile  -t local/yakman-frontend

install: # Run PNPM install in the frontend project
	@cd frontend && pnpm install

bd: ## (backend-dev) Start the backend in dev mode
	@cd backend && cargo run

fd: ## (frontend-dev) Start the fronend in dev mode
	@cd frontend && pnpm run dev

bf: ## cargo fmt
	@cd backend && cargo fmt

bfc: ## cargo fmt --check
	@cd backend && cargo fmt --check