# SplitVibe Rust -- Technical Specification

**Version:** 0.1
**Date:** 2026-03-07
**Status:** Draft

---

## 1. Overview

This document captures the technology choices and architecture for the SplitVibe Rust rewrite. All decisions are made in the context of a small-to-medium personal/family app with a primary deployment target of Microsoft Azure and a strong requirement for efficient local development.

---

## 2. Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Backend** | Actix-web 4.x | High-performance async web framework with mature middleware ecosystem. |
| **Frontend** | Leptos 0.7.x (SSR + WASM hydration) | Full-stack Rust framework; SSR for initial load, WASM hydration for interactivity. |
| **Integration** | leptos_actix | Bridges Leptos SSR into Actix-web server. |
| **Language** | Rust (nightly) | Memory safety, performance, single language for frontend + backend. Nightly required by Leptos. |
| **UI** | Tailwind CSS | Utility-first styling, same approach as original SplitVibe. |
| **Auth** | oauth2 crate + PKCE | Google OAuth with PKCE flow; mock login for development. |
| **Sessions** | actix-session + PostgreSQL | Server-side sessions stored in database for easy revocation. |
| **Database** | PostgreSQL + SQLx 0.8.x | Compile-time checked queries, async, migration tooling. |
| **File Storage** | Azure Blob Storage (azure_storage_blobs) | Native Azure object storage with signed-URL access for attachments. |
| **FX Rates** | Frankfurter API | Free, no API key, ECB-backed exchange rate data; fetched and cached daily. |

---

## 3. Project Structure

Cargo workspace with three crates for clean separation of concerns:

```
splitvibe-rust/
  Cargo.toml                     # workspace root
  rust-toolchain.toml            # pin nightly
  crates/
    splitvibe-core/              # pure business logic (no async, no DB)
      src/lib.rs                 # split calculation, balance simplification
    splitvibe-db/                # SQLx models, queries, migrations
      src/lib.rs                 # model structs, query functions
      migrations/                # SQLx migration files
    splitvibe-server/            # Actix-web + Leptos binary
      src/
        main.rs                  # server entrypoint
        app.rs                   # Leptos App component + routes
        auth/                    # OAuth2, sessions, middleware, extractors
        pages/                   # Leptos SSR page components
        components/              # Leptos reusable UI components
        storage.rs               # Azure Blob SAS URL generation
        error.rs                 # Error types
      style/main.css             # Tailwind CSS
  Dockerfile                     # multi-stage build with cargo-leptos
  docker-compose.yml             # local Postgres + Azurite
  .github/workflows/ci.yml      # CI pipeline
  infra/                         # Bicep IaC templates
```

---

## 4. Local Development

All backing services run as Docker containers via Docker Compose.

### Services (docker-compose.yml)

| Service | Image | Purpose |
|---------|-------|---------|
| `db` | `postgres:16-alpine` | Local PostgreSQL instance |
| `storage` | `mcr.microsoft.com/azure-storage/azurite` | Azure Blob Storage emulator |

The Rust application runs natively via `cargo leptos serve` for fast iteration with hot reload.

### Workflow

```bash
docker compose up -d           # Start backing services
sqlx migrate run               # Apply database migrations
cargo leptos serve              # Start dev server (port 3000)
```

---

## 5. Deployment Architecture (Azure)

```
GitHub
  |
  |  push to main
  v
GitHub Actions
  |-- Run tests (cargo test with Postgres service container)
  |-- Build Docker image (multi-stage with cargo-leptos)
  |-- Push to Azure Container Registry (ACR)
  '-- Deploy to Azure Container Apps
            |
            |-- Actix-web + Leptos App (Container App, port 8080)
            |     |-- Serves UI (SSR + WASM hydration)
            |     '-- Handles API requests + server functions
            |
            |-- Azure Database for PostgreSQL -- Flexible Server
            |
            '-- Azure Blob Storage (attachments)
```

### Azure Resources

| Resource | Purpose |
|----------|---------|
| **Azure Container Registry** | Stores Docker images built by CI |
| **Azure Container Apps** | Hosts the application; scales zero-to-one |
| **Azure Database for PostgreSQL -- Flexible Server** | Managed Postgres; private VNet access |
| **Azure Blob Storage** | Expense attachments; served via short-lived signed URLs |
| **Azure Key Vault** | Stores secrets (DB password, OAuth credentials) |
| **Log Analytics** | Container App logging and diagnostics |

### Scaling & Cost

- Container Apps scales to zero replicas when there is no traffic.
- PostgreSQL Flexible Server uses the Burstable tier (lowest cost) for v1.

---

## 6. CI/CD (GitHub Actions)

### `ci.yml` -- runs on every pull request
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test` (with Postgres service container)
4. `cargo leptos build --release` (smoke-check)

### `deploy.yml` -- runs on merge to `main`
1. Build multi-stage Docker image
2. Push to Azure Container Registry
3. Deploy new revision to Azure Container Apps
4. Run `sqlx migrate run` against production database

OIDC federation for Azure auth (no stored credentials in GitHub).

---

## 7. Authentication Flow

1. User clicks "Sign in with Google".
2. `oauth2` crate handles the PKCE authorization code flow.
3. On callback, the app exchanges the code for tokens and fetches user info.
4. On first login, a `User` record is created in PostgreSQL.
5. An `actix-session` session is created and stored in PostgreSQL.
6. `AuthenticatedUser` Actix extractor reads the session and returns the user or 401.
7. Mock login (`POST /auth/mock-login`) available when `MOCK_AUTH_ENABLED=true`.

---

## 8. File Attachment Flow

**Upload:**
1. Client requests a pre-signed upload URL from the API (`POST /api/attachments/presign`).
2. API generates a short-lived Azure Blob Storage SAS URL and returns it.
3. Client uploads the file directly to Blob Storage (no file bytes pass through the app server).
4. Client notifies the API of the completed upload; API saves the blob reference to the database.

**Download:**
1. Client requests a download URL from the API (`GET /api/attachments/:id`).
2. API verifies the requester is a member of the relevant group, then returns a short-lived SAS read URL.
3. Client fetches the file directly from Blob Storage.

---

## 9. FX Rate Caching

- A background task calls the Frankfurter API once per day.
- Rates are stored in an `exchange_rates` table in Postgres (`from_ccy`, `to_ccy`, `rate`, `date`).
- When an expense is created, the current cached rate is looked up and stored directly on the expense record -- it is never changed afterwards.

---

## 10. Key Design Decisions

- **Hybrid API**: Leptos server functions for SSR pages + thin REST layer at `/api/` for programmatic access.
- **Sessions over JWT**: Server-side PostgreSQL sessions via `actix-session` for easy revocation.
- **Decimal as strings in JSON**: `rust_decimal` serializes as strings to preserve precision.
- **IDs**: `cuid2` crate to match original Prisma CUID generation.
- **Health check**: `GET /api/health` returns 200 for Container Apps probes.

---

## 11. Environment Variables

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | Postgres connection string |
| `SESSION_SECRET` | Secret for signing session cookies |
| `GOOGLE_CLIENT_ID` | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | Google OAuth client secret |
| `GOOGLE_REDIRECT_URL` | Google OAuth redirect URL |
| `AZURE_STORAGE_ACCOUNT_NAME` | Blob Storage account name |
| `AZURE_STORAGE_ACCOUNT_KEY` | Blob Storage account key |
| `AZURE_STORAGE_CONTAINER_NAME` | Blob container name for attachments |
| `AZURE_STORAGE_CONNECTION_STRING` | Full connection string (Azurite in dev) |
| `MOCK_AUTH_ENABLED` | Enable mock login (dev only) |

---

## 12. Key Crates

| Crate | Purpose |
|-------|---------|
| `actix-web` | HTTP server |
| `leptos` | Full-stack UI framework |
| `leptos_actix` | Leptos + Actix-web integration |
| `leptos_router` | Client-side routing |
| `sqlx` | Async database access with compile-time checks |
| `oauth2` | OAuth 2.0 client |
| `reqwest` | HTTP client (for OAuth token exchange, FX API) |
| `rust_decimal` | Arbitrary-precision decimal for money |
| `serde` | Serialization/deserialization |
| `chrono` | Date/time handling |
| `cuid2` | CUID v2 ID generation |
| `actix-session` | Session management |
| `azure_storage_blobs` | Azure Blob Storage client |
| `tokio` | Async runtime |
| `tracing` | Structured logging |
| `validator` | Input validation |
| `thiserror` | Error type derivation |
| `dotenvy` | .env file loading |
