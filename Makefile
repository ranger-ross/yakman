help: ## Shows help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

redis: ## Starts Redis with Docker
	docker run -it --rm -p 6379:6379 redis

postgres: ## Starts Postgres with Docker
	docker run -it --rm -e POSTGRES_PASSWORD=password -p 5432:5432 postgres

postgres-connect: ## Connect to Postgres Docker container with psql
	PGPASSWORD=password psql --host localhost --username postgres

leptos: ## Starts the YakMan frontend with trunk
	cd legacy_frontend; trunk serve --open

watch-tailwind: ## Runs Tailwind to update css
	cd legacy_frontend; npx tailwind -o style/output.css -w

fmt: ## Runs leptosfmt to format the frontend view macros (this may cause Trunk to bug out for 30 seconds)
	leptosfmt legacy_frontend

build-backend: ## Builds the backend dockerfile with tag local/yakman-backend
	docker build . -f backend.Dockerfile -t local/yakman-backend

build-frontend: ## Builds the frontend dockerfile with tag local/yakman-frontend
	docker build . -f frontend.Dockerfile  -t local/yakman-frontend