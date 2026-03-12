# SplitVibe Rust -- Claude Code Guide

## Repository

https://github.com/marcgs/SplitVibe-Rust-v2

---

## Project Overview

SplitVibe is a shared-expense tracking app. Users create groups, add expenses (with flexible split modes), and settle up balances. It supports multi-currency expenses with FX rate capture at creation time and Azure Blob Storage for receipt attachments.

**Stack:** Actix-web 4.x . Leptos 0.7.x (SSR + WASM hydration) . Tailwind CSS . SQLx 0.8.x + PostgreSQL . oauth2 crate (Google OAuth + dev mock login) . Azure Blob Storage . Playwright (E2E)

---

## TDD Workflow

**Always write failing tests first.**

1. Write a failing test that describes the desired behavior
2. Implement the minimal code to make it pass
3. Refactor while keeping tests green
4. Never skip tests -- all features must have coverage

---

## Commands

### Development
```bash
docker compose up -d       # Start backend services (postgres + azurite)
cargo leptos serve          # Start dev server with hot reload (port 3000)
cargo leptos build --release # Production build
sqlx migrate run            # Run database migrations
```

### Testing
```bash
cargo test                             # Run all unit/integration tests
cargo test -p splitvibe-core           # Run core crate tests only
cargo test -p splitvibe-db             # Run db crate tests only
cargo test -p splitvibe-server         # Run server crate tests only
cargo test -- test_name_pattern        # Run tests matching a name
```

### Quality
```bash
cargo clippy -- -D warnings  # Lint (deny all warnings)
cargo fmt --check             # Format check
```

> **Auto-hook:** After every `Edit` or `Write` on a `.rs` file, the PostToolUse hook in `.claude/hooks/lint-clippy.sh` automatically runs `cargo clippy` and `cargo fmt --check` on the changed file. Lint errors will surface immediately after edits.

---

## Mock Users

Development mode uses three named mock users. **All three must always be available on the login page:**

| Name | ID | Avatar |
|------|-----|--------|
| **Alice** | Deterministic cuid2 or hardcoded | Distinct avatar URL |
| **Bob** | Deterministic cuid2 or hardcoded | Distinct avatar URL |
| **Charlie** | Deterministic cuid2 or hardcoded | Distinct avatar URL |

Each mock user must have a unique `id`, `display_name`, and `avatar_url`. The login page must show three separate buttons: "Login as Alice", "Login as Bob", "Login as Charlie".

---

## `.env.example` Consistency

- **Every environment variable** used by the app must be present in `.env.example` with a valid default value.
- `SESSION_SECRET` must be >= 64 bytes in `.env.example`.
- **Smoke test:** `cp .env.example .env && docker compose up -d && cargo leptos serve` must work without manual edits.
- When adding or changing env vars in code, **always update `.env.example`** in the same commit.

---

## File Structure

```
splitvibe-rust/
  Cargo.toml                     # workspace root
  rust-toolchain.toml            # pin nightly (Leptos requirement)
  crates/
    splitvibe-core/              # pure business logic (no async, no DB)
      src/lib.rs
    splitvibe-db/                # SQLx models, queries, migrations/
      src/lib.rs
      migrations/                # SQLx migrations
    splitvibe-server/            # Actix-web + Leptos binary
      src/
        main.rs
        app.rs                   # Leptos App + routes
        auth/                    # OAuth2, sessions, middleware
        pages/                   # Leptos SSR page components
        components/              # Leptos reusable UI components
        storage.rs               # Azure Blob SAS URLs
        error.rs
      style/main.css             # Tailwind CSS
  Dockerfile
  docker-compose.yml             # local Postgres + Azurite
  .github/workflows/ci.yml
  infra/                         # Bicep templates
  docs/
    plan.md                      # Implementation plan
    spec.md                      # Product specification
    tech.md                      # Technical architecture
    backlog.md                   # Story backlog
```

---

## Environment Variables

Copy `.env.example` to `.env` for local development. Docker Compose injects backing services automatically.

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
| `MOCK_AUTH_ENABLED` | Set to `true` to enable mock login (dev only) |

Local Azurite credentials are hardcoded in `docker-compose.yml` (standard emulator defaults).

---

## Key Documentation

- `docs/spec.md` -- Product requirements and domain rules
- `docs/tech.md` -- Architecture, auth flow, deployment, env vars
- `docs/backlog.md` -- Story backlog with priorities

---

## Coding Conventions

- Use `cargo clippy` and `cargo fmt` before considering work done
- Follow existing patterns in the codebase for consistency
- Use `rust_decimal::Decimal` for all monetary amounts
- Use `cuid2` for generating IDs
- Use `thiserror` for error types, `tracing` for logging
- Prefer compile-time checked SQLx queries where possible
- Leptos server functions for SSR pages, thin REST layer at `/api/` for programmatic access
