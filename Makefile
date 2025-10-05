lint:
	cargo fmt --check && cargo clippy --target wasm32-unknown-unknown -- -D warnings

run:
	@echo "Starting services..."
	docker compose up --build
	@echo "Services started."

contact-message:
	./scripts/test-contact.sh

db-inspect:
	@echo "Inspecting local D1 database..."
	@docker exec -it contact-us-worker-rs-worker-1 npx wrangler d1 execute contact-us-worker-rs --local --command "SELECT * FROM contact_messages ORDER BY created_at DESC;"