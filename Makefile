.PHONY: dev db db-stop migrate seed test build clean

# Start PostgreSQL in Docker
db:
	docker compose up -d postgres
	@echo "Waiting for PostgreSQL..."
	@until docker compose exec postgres pg_isready -U ideaforge > /dev/null 2>&1; do sleep 1; done
	@echo "PostgreSQL is ready."

# Stop PostgreSQL
db-stop:
	docker compose down

# Run database migrations
migrate:
	cd src && cargo run --bin migrate

# Seed the database with sample data
seed:
	cd src && cargo run --bin seed

# Build the project
build:
	cd src && cargo build

# Run all tests
test:
	cd src && cargo test

# Start the dev server (starts DB, migrates, seeds, runs server)
dev: db
	cd src && cargo run --bin ideaforge

# Clean build artifacts
clean:
	cd src && cargo clean
