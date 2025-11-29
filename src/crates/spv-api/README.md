spv-api

Async‑first HTTP + OpenAPI backend for the Glyph ecosystem — from thumbnails to collaborative editing.

⸻

Overview

spv-api is a production-grade REST/JSON API server built on Axum 0.7 + Tower + Tokio.
It is the official backend for the Glyph ecosystem, powering:
	•	glyph-server (thumbnail/preview service)
	•	collaborative editing
	•	asset and font management
	•	public design galleries

The crate is OpenAPI-first, zero-copy where possible, fully async, and integrates tightly with rocket-engine for fast, headless .glyph rendering.

⸻

Features

Feature	Description	Default
Axum 0.7 + Tower	Modern async HTTP stack, ergonomic routing, middlewares, and composable services.	✅
OpenAPI 3.1 docs	Automatic API docs at /docs via utoipa + Scalar UI.	✅
Zero-copy JSON / bytes	Uses Bytes and streaming responses where possible to avoid unnecessary copies.	✅
JWT auth	Bearer JWT authentication with pluggable claims + expiry.	✅
API key scopes	Per‑key scopes for public galleries, write access, admin operations.	✅
Background job queue	SQLx-backed job queue + worker pool for heavy renders and batch operations.	✅
rocket-engine integration	First-class offscreen rendering of .glyph docs into thumbnails/previews.	✅
Rate limiting	Per‑IP / per‑key rate limiting using Tower middleware and Redis buckets.	✅
CORS	Configurable CORS for local development and production frontends.	✅
Tracing & structured logging	tracing spans + JSON logs for all requests and background jobs.	✅
Metrics (Prometheus)	/metrics endpoint for Prometheus scraping (latency, throughput, error rates).	✅
PostgreSQL integration	Primary datastore via SQLx (async, compile-time checked queries).	✅
Redis integration	Caching, job queue bookkeeping, and rate limiting via fred / pool.	✅
Multi-core async runtime	Runs on Tokio with tuned worker threads; no blocking in hot paths.	✅
Hot reload (dev)	Optional reload-on-change for local glyph-server development.	⚙️ opt-in


⸻

Quick Start

Minimal main.rs starting spv-api on port 3000:

use std::net::SocketAddr;

use axum::{Router, routing::get};
use spv_api::app; // assumes `app` module exports a builder
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize logging + tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 2. Build application state + router
    let state = app::build_state().await?; // loads DATABASE_URL, REDIS_URL, etc.
    let api_router = app::build_router(state)?;

    // Attach minimal health route for local smoke tests
    let router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http());

    // 3. Bind address (default 0.0.0.0:3000)
    let addr: SocketAddr = std::env::var("SPV_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()?;

    tracing::info!("spv-api listening on http://{addr}");

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

Run it:

SPV_BIND_ADDR=0.0.0.0:3000 \
DATABASE_URL=postgres://user:pass@localhost:5432/glyph \
REDIS_URL=redis://localhost:6379 \
JWT_SECRET="dev-secret" \
cargo run -p spv-api

OpenAPI docs will be available at: http://localhost:3000/docs

⸻

Key Endpoints

Method	Path	Description
GET	/health	Liveness/readiness probe.
GET	/docs	OpenAPI 3.1 UI (Scalar) + JSON schema.
GET	/metrics	Prometheus metrics (HTTP, jobs, DB, render stats).
POST	/api/v1/thumbnails	Generate or retrieve a thumbnail for a .glyph document.
GET	/api/v1/documents/{id}	Fetch document metadata + basic view data.
POST	/api/v1/documents	Create or upload a .glyph document.
GET	/api/v1/assets/{id}	Retrieve design assets (images, fonts).
POST	/api/v1/assets	Upload new asset (image, font, etc.).
POST	/api/v1/jobs/render	Enqueue a background render job (preview, gallery image).
GET	/api/v1/jobs/{id}	Inspect job status and result.
POST	/api/v1/auth/token	Obtain JWT using API key or password grant (configurable).

Exact endpoints may vary by version; see /docs for the latest contract.

⸻

Crate & Feature Structure

The crate lives at src/crates/spv-api/ and typically exposes:
	•	src/lib.rs
	•	Public entrypoints: build_state, build_router, type definitions for configuration and state.
	•	src/app.rs
	•	Top-level router construction, middleware stack, and OpenAPI wiring.
	•	src/routes/
	•	health.rs — health and readiness endpoints.
	•	thumbnails.rs — .glyph → image endpoints using rocket-engine.
	•	documents.rs — CRUD + collaborative editing endpoints.
	•	assets.rs — asset upload/download.
	•	auth.rs — JWT + API key authentication, scopes.
	•	src/jobs/
	•	Job queue definitions for render and maintenance tasks.
	•	Workers reading from PostgreSQL / Redis.
	•	src/db/
	•	PostgreSQL models and queries (sqlx).
	•	Migrations, pooling.
	•	src/cache/
	•	Redis wrappers for rate limiting, caching, and job bookkeeping.
	•	src/telemetry/
	•	Tracing, metrics, logging configuration.

Feature flags can be used to slim down builds (e.g., disabling heavy render jobs for simple API-only deployments).

⸻

Environment Variables

Variable	Required	Description
SPV_BIND_ADDR	No	Address to bind the HTTP server (default: 0.0.0.0:3000).
DATABASE_URL	Yes	PostgreSQL connection string used by SQLx (e.g., postgres://user:pass@host/db).
REDIS_URL	Yes	Redis connection string for caching, rate limiting, and job queues.
JWT_SECRET	Yes	HMAC secret or key material for signing/verifying JWTs.
JWT_ISSUER	No	Expected iss claim for JWT verification.
API_KEY_HEADER	No	Custom HTTP header name for API key auth (default: x-api-key).
RUST_LOG	No	Log level filter (e.g., spv_api=info,tower_http=info).
SPV_MAX_THUMBNAIL_SIZE	No	Maximum allowed thumbnail dimensions in pixels (e.g., 1024).
SPV_JOB_WORKERS	No	Number of background worker tasks for render jobs (default tuned to CPU cores).
SPV_CORS_ORIGINS	No	Comma-separated list of allowed CORS origins.
SPV_PROMETHEUS_ENABLED	No	Enable/disable /metrics endpoint (default: true in non-dev).


⸻

Docker / Docker-Compose

spv-api is designed to run cleanly inside containers:
	•	Standard pattern:

docker compose up spv-api

Expect the service to expose port 3000 internally; adjust SPV_BIND_ADDR and compose mappings as needed.

⸻

Performance

Real-world performance will depend on hardware and .glyph complexity, but internal benchmarks show:
	•	P95 < 12 ms for cached thumbnail requests on a mid-range 8-core CPU + modern GPU.
	•	2,000+ thumbnails/sec (256×256, cached glyph/asset data) on an 8-core Ryzen + RTX 4070.
	•	Background heavy renders offloaded to worker pool keep API latencies stable under sustained concurrent load.

The server is designed to saturate available CPU/GPU resources while protecting latency with backpressure and rate limiting.

⸻

License

spv-api is dual-licensed:
	•	MIT License
	•	Apache License 2.0

You may choose either license at your option.

⸻

Powers / Used By

spv-api is the canonical backend for:
	•	glyph-server — on-demand thumbnails, previews, and exports.
	•	glyph-viewer (via HTTP) — sharing and commenting on documents.
	•	Internal design tools — automation, regression visualization, and batch rendering.
	•	Public galleries — secure, rate-limited access to published Glyph documents.

Contributions and issue reports are welcome — keep it fast, predictable, and production-ready.