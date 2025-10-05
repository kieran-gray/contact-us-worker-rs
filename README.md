# Contact Us Worker (Rust)

A basic and extendable contact-us form submission endpoint written in Rust and compiled to WebAssembly (WASM) to run on Cloudflare Workers. Handles form submissions with bot protection via Cloudflare Turnstile and stores messages in D1 database.

## Architecture

The project follows a clean architecture pattern with separation of concerns:

- `api/` - HTTP routing, request/response schemas, and CORS handling
- `application/` - Business logic and service layer
- `domain/` - Core entities, enums, and repository interfaces
- `infrastructure/` - Database and external service implementations
- `setup/` - Configuration and application state

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [Node.js](https://nodejs.org/) and npm (for testing)

## Local Development

1. **Create environment file:**

   Copy the example environment file (uses a Turnstile secret that always passes):
   ```bash
   cp example.dev.vars .dev.vars
   ```

   Or create your own `.dev.vars` file with your Cloudflare Turnstile secret:
   ```bash
   TURNSTILE_SECRET_KEY="your-cloudflare-turnstile-secret-key"
   ```

2. **Run the project:**
   ```bash
   make run
   ```

   This will build and start the worker in a Docker container.

3. **Test the endpoint:**
   ```bash
   make contact-message
   ```

   This will send a test message to the contact-us endpoint.

4. **Inspect the database:**
   ```bash
   make db-inspect
   ```

   This will show all contact messages stored in the local D1 database.

## Testing & Linting

```bash
# Run tests
npm test

# Lint code
make lint
```

## Production Deployment

### Prerequisites
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)
- Cloudflare account with Workers and D1 enabled

### Setup

1. **Install dependencies:**
   ```bash
   npm install
   cargo install worker-build
   ```

2. **Configure database:**
   - Create a D1 database in your Cloudflare dashboard
   - Update `wrangler.toml` with your database ID
   - Run migrations:
     ```bash
     wrangler d1 migrations apply contact-us-worker-rs
     ```

3. **Configure environment:**
   - Update `ALLOWED_ORIGINS` in `wrangler.toml`
   - Add your Cloudflare Turnstile secret key:
     ```bash
     wrangler secret put TURNSTILE_SECRET_KEY
     ```

### Deploy

```bash
# Deploy to production
wrangler deploy

# Deploy to dev environment
wrangler deploy --env dev
```

## API Endpoints

### Health Check
```
GET /api/v1/health-check/
```

### Submit Contact Message
```
POST /api/v1/contact-us/
Content-Type: application/json

{
  "token": "cloudflare-turnstile-token",
  "category": "general",
  "email": "user@example.com",
  "name": "John Doe",
  "message": "Your message here",
  "data": {} // Optional additional data
}
```

## Extending

The modular architecture makes it easy to:
- Add new endpoints in `src/api/routes/`
- Implement additional validation in `src/application/request_validation_service.rs`
- Create new categories in `src/domain/enums.rs`
- Add database migrations in `migrations/`

## License

See [LICENSE](LICENSE) file for details.
