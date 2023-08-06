help: ## Shows help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

redis: ## Starts Redis with Docker
	docker run -it --rm -p 6379:6379 redis

postgres: ## Starts Postgres with Docker
	docker run -it --rm -e POSTGRES_PASSWORD=password -p 5432:5432 postgres

postgres-connect: ## Connect to Postgres Docker container with psql
	PGPASSWORD=password psql --host localhost --username postgres

build-backend: ## Builds the backend dockerfile with tag local/yakman-backend
	docker build . -f backend.Dockerfile -t local/yakman-backend

build-frontend: ## Builds the frontend dockerfile with tag local/yakman-frontend
	docker build . -f frontend.Dockerfile  -t local/yakman-frontend