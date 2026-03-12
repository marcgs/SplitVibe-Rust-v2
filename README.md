# SplitVibe

A shared-expense tracking app built with Rust. Users create groups, add expenses with flexible split modes, track who owes whom, and settle up balances.

## Tech Stack

- **Backend:** Actix-web 4.x
- **Frontend:** Leptos 0.7.x (SSR + WASM hydration)
- **Database:** PostgreSQL + SQLx 0.8.x
- **Auth:** Google OAuth + mock login for development

## Prerequisites

- [Rust (nightly)](https://rustup.rs/) — installed automatically via `rust-toolchain.toml`
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) — `cargo install cargo-leptos`
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- WASM target — `rustup target add wasm32-unknown-unknown`

## Setup

```bash
# Clone the repository
git clone https://github.com/marcgs/SplitVibe-Rust-v2.git
cd SplitVibe-Rust-v2

# Copy environment variables
cp .env.example .env

# Start backing services (PostgreSQL + Azurite)
docker compose up -d

# Start the dev server with hot reload
cargo leptos serve
```

## Access the App

Open [http://localhost:3000](http://localhost:3000) in your browser.

Three mock users (Alice, Bob, Charlie) are available on the login page for development.

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p splitvibe-core
cargo test -p splitvibe-db
cargo test -p splitvibe-server
```

## Code Quality

```bash
# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check
```
