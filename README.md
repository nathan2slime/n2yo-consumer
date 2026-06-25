# n2yo-consumer

Rust monorepo for the n2yo-consumer project. The repository currently contains a single stateless HTTP service built with Actix Web that consumes the N2YO REST API v1 and exposes documented satellite endpoints.

## Current Stack

- Rust edition 2024
- Actix Web
- Docker Compose
- Reqwest with rustls for outbound N2YO calls
- Utoipa for OpenAPI generation
- `env_logger` and `log` for logging

## Current Structure

- `service/`: main HTTP service
- `docker-compose.yml`: local API runtime
- `.env.example`: root environment example for Docker Compose
- `.env.production`: root environment file used by Docker Compose

## What Is Implemented Today

The API currently provides:

- `GET /health`
- `GET /satellite/tle/{id}`
- `GET /satellite/positions/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{seconds}`
- `GET /satellite/visualpasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_visibility}`
- `GET /satellite/radiopasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_elevation}`
- `GET /satellite/above/{observer_lat}/{observer_lng}/{observer_alt}/{search_radius}/{category_id}`
- `GET /service-docs/openapi.json`
- Swagger UI at `GET /docs/`

## Requirements

- Rust toolchain compatible with edition 2024
- Cargo
- Docker and Docker Compose, optionally

## Environment Variables

The project uses root `.env` files for local and Docker-based configuration.

Main application variables:

- `API_BIND`: API bind address
- `N2YO_API_KEY`: N2YO REST API key
- `N2YO_BASE_URL`: N2YO satellite API base URL
- `N2YO_TIMEOUT_SECONDS`: outbound request timeout in seconds
- `RUST_LOG`: log configuration

Local development example:

```bash
API_BIND=0.0.0.0:8080
N2YO_API_KEY=change-me-n2yo-api-key
N2YO_BASE_URL=https://api.n2yo.com/rest/v1/satellite
N2YO_TIMEOUT_SECONDS=10
RUST_LOG=info,actix_web=info
```

## Running Locally

### 1. Configure the environment

```bash
cp .env.example .env
```

Set `N2YO_API_KEY` before starting the API.

### 2. Run the API with Cargo

```bash
cargo run -p service
```

The API will be available at `http://localhost:8080`.

### 3. Optional: run with Docker Compose

```bash
docker compose up -d --build
```

This starts the `service` container only.

## Useful Commands

### Development

```bash
cargo run -p service
```

### Build

```bash
cargo build -p service
cargo build -p service --release
```

### Tests

```bash
cargo test -p service
```

The test suite does not require external services and does not call N2YO.

### Compile Check

```bash
cargo check -p service
```

### Local Service with Docker

```bash
docker compose up -d --build service
docker compose down
```

## Current Endpoints

- `GET /health`: checks API process health without spending N2YO transactions
- `GET /satellite/tle/{id}`: retrieves TLE data for a NORAD satellite id
- `GET /satellite/positions/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{seconds}`: retrieves future positions, limited to 300 seconds
- `GET /satellite/visualpasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_visibility}`: retrieves visual passes, limited to 10 prediction days
- `GET /satellite/radiopasses/{id}/{observer_lat}/{observer_lng}/{observer_alt}/{days}/{min_elevation}`: retrieves radio passes, limited to 10 prediction days
- `GET /satellite/above/{observer_lat}/{observer_lng}/{observer_alt}/{search_radius}/{category_id}`: retrieves satellites above an observer, with radius from 0 to 90 degrees
- `GET /service-docs/openapi.json`: returns the OpenAPI specification as JSON
- `GET /docs/`: serves Swagger UI

## N2YO Notes

- All satellite data is fetched from `https://api.n2yo.com/rest/v1/satellite/` by default.
- The N2YO API key is kept server-side in `N2YO_API_KEY` and is not accepted from client requests.
- N2YO transaction limits still apply to this consumer service.
- The `/health` endpoint does not call N2YO to avoid spending transactions.

## Docker

The current `docker-compose.yml` defines:

- `service`: build and runtime for the Rust API

The [service/Dockerfile](/home/nathan3boss/projects/n2yo-consumer/service/Dockerfile) builds a release binary and runs the final artifact in a slim Debian image.

## GitHub Actions

The workflow at `.github/workflows/actions.yml` runs formatting, clippy, tests, builds `cargo build --release -p service`, and uploads the generated `target/release/service` binary as an artifact named `service-linux-x86_64`.

## Project Status

The repository is currently focused on a stateless N2YO consumer API for n2yo-consumer.
