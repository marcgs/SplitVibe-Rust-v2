# SplitVibe Rust Rewrite -- Implementation Plan

## Context

SplitVibe is a collaborative expense-tracking app originally built with Next.js/React/Prisma/PostgreSQL on Azure. This plan covers a full Rust rewrite targeting the MVP scope (stories 0-8). The goal is a production-ready app deployed on Azure Container Apps.

**Workflow**: GitHub-driven with issues containing end-user acceptance criteria. Each story is implemented on a feature branch, submitted as a PR, and validated before merge.

**Repo**: https://github.com/marcgs/SplitVibe-Rust-v2

## Tech Stack

- **Backend**: Actix-web 4.x
- **Frontend**: Leptos 0.7.x (SSR + WASM hydration) via `leptos_actix`
- **Database**: PostgreSQL with SQLx 0.8.x (compile-time checked queries)
- **Auth**: `oauth2` crate (Google OAuth + dev mock login)
- **Storage**: `azure_storage_blobs` for presigned URL uploads
- **Deployment**: Azure Container Apps, Bicep IaC, GitHub Actions CI/CD

## Project Structure

```
splitvibe-rust/
  Cargo.toml                     # workspace root
  rust-toolchain.toml            # pin nightly (Leptos requirement)
  crates/
    splitvibe-core/              # pure business logic (no async, no DB)
    splitvibe-db/                # SQLx models, queries, migrations/
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
```

## MVP Stories

| # | Story | Priority | Dependencies |
|---|-------|----------|-------------|
| 0 | Project Scaffold & Dev Environment | P0 | None |
| 1 | Database Schema & Migrations | P0 | Story 0 |
| 2 | Authentication (Mock + Google OAuth) | P0 | Story 0, 1 |
| 3 | Group Management | P1 | Story 2 |
| 4 | Expense Creation (Equal Split) | P1 | Story 3 |
| 5 | Balance Calculation & Display | P1 | Story 4 |
| 6 | Settlement Recording | P1 | Story 5 |
| 7 | Azure Infrastructure (Bicep) | P2 | Story 0 |
| 8 | CI/CD Pipeline | P2 | Story 0 |

## Execution Workflow

For each story:

1. **Implement**: Use the `implement` skill to work on a feature branch, referencing the GitHub issue
2. **PR**: Create a pull request with the implementation
3. **Validate**: Use the `validate-pr` skill to verify acceptance criteria are met
4. **Merge**: Merge the PR once validation passes

Stories 1 and 2 can be parallelized. Stories 3-6 are sequential. Stories 7-8 can start after Story 0.

## Key Design Decisions

- **Hybrid API**: Leptos server functions for SSR pages + thin REST layer at `/api/` for programmatic access
- **Sessions over JWT**: Server-side PostgreSQL sessions via `actix-session`
- **Decimal as strings in JSON**: `rust_decimal` serializes as strings to preserve precision
- **IDs**: `cuid2` crate to match original Prisma CUID generation

## Key Crates

`actix-web`, `leptos`, `leptos_actix`, `leptos_router`, `sqlx`, `oauth2`, `reqwest`, `rust_decimal`, `serde`, `chrono`, `cuid2`, `actix-session`, `azure_storage_blobs`, `tokio`, `tracing`, `validator`, `thiserror`, `dotenvy`
