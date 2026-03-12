# SplitVibe Rust -- Story Backlog

**Status:** MVP scope (Stories 0-8)

---

## MVP Stories

### Story 0 - Project Scaffold & Dev Environment
**Priority:** P0 (prerequisite for all other stories)
**Labels:** `mvp`, `story-0`

Set up the Rust workspace, dev tooling, and local development environment so contributors can clone, build, and run the app.

**Dependencies:** None

**Acceptance Criteria:**

1. Given a fresh clone, when running `cp .env.example .env && docker compose up -d && cargo leptos serve`, then `http://localhost:3000` returns HTTP 200 with a "Welcome to SplitVibe" heading.
2. Given the app is running, when the page loads, then zero JS errors in the console and WASM hydration loads successfully.
3. Given a fresh clone, when running `cargo test`, then all tests pass.
4. Given a fresh clone, when running `cargo clippy -- -D warnings && cargo fmt --check`, then both exit 0.
5. Given the login page, when viewing mock login options, then three users are available: Alice, Bob, Charlie.
6. Given a fresh clone, when reading `README.md`, then it contains: description, prerequisites, setup instructions, how to run tests, how to access the app.
7. Given `.env.example`, when copied to `.env` without changes, then the app starts without errors (SESSION_SECRET >= 64 bytes).
8. **Error path:** Given the app is running, when navigating to `/does-not-exist`, then a 404 page is returned.

---

### Story 1 - Database Schema & Migrations
**Priority:** P0
**Labels:** `mvp`, `story-1`

Create the database schema via SQLx migrations so the app has persistent storage for users, groups, expenses, settlements, and related data.

**Dependencies:** Story 0

**Acceptance Criteria:**

1. Given a running Postgres container and `.env` from `.env.example`, when running `sqlx migrate run`, then migrations apply with exit 0.
2. Given migrations applied, then tables exist: `users`, `groups`, `group_members`, `expenses`, `expense_payers`, `expense_splits`, `settlements`, `sessions`.
3. Given migrations applied, then `expenses.amount` uses `DECIMAL(14,4)`.
4. Given migrations applied, then foreign keys: `expenses.group_id -> groups.id`, `group_members.user_id -> users.id`, `settlements.group_id -> groups.id`.
5. Given a fresh DB, when running `cargo test -p splitvibe-db`, then at least one test verifies migrations apply and tables are queryable.
6. **Error path:** Given migrations already applied, when running `sqlx migrate run` again, then it completes without error (idempotent).

---

### Story 2 - Authentication (Mock + Google OAuth)
**Priority:** P0
**Labels:** `mvp`, `story-2`

Users can sign in to SplitVibe using their Google account. A mock login is available for development/testing.

**Dependencies:** Story 0, Story 1

**Acceptance Criteria:**

1. Given an unauthenticated user, when navigating to `/groups`, then redirected to `/auth/login`.
2. Given the login page, then there is "Sign in with Google" and three buttons: "Login as Alice", "Login as Bob", "Login as Charlie".
3. Given the login page, when Alice clicks "Login as Alice", then redirected to `/groups` and navbar shows "Alice".
4. Given Alice is logged in, when she clicks "Sign out", then redirected to `/auth/login` and `/groups` redirects back to login.
5. Given Alice is logged in and refreshes, then session is preserved.
6. Given the login page, when clicking "Sign in with Google" without credentials configured, then a clear error message (not panic/500).
7. Zero JS console errors during login/redirect/logout flows.
8. **Error path:** Given Alice is logged in, when submitting mock login for Bob, then session switches cleanly -- no corruption.

**Note:** Google OAuth with real credentials -> BLOCKED if not configured. Ask user.

---

### Story 3 - Group Management
**Priority:** P1
**Labels:** `mvp`, `story-3`

Users can create expense groups, invite others via a shareable link, and see all their groups.

**Dependencies:** Story 2

**Acceptance Criteria:**

1. Given Alice is logged in, when she creates group "Trip to Paris", then redirected to group detail showing "Trip to Paris" with Alice as member.
2. Given Alice created "Trip to Paris", when navigating to `/groups`, then list shows "Trip to Paris" with "1 member".
3. Given Alice is on group detail, when clicking "Copy invite link", then URL has format `/join/<token>`.
4. Given Bob is logged in and opens the invite link, then Bob is added and group shows Alice and Bob as members.
5. Given Bob is already a member, when opening the invite link again, then "Already a member" message, not added twice.
6. Given Charlie is not logged in and opens an invite link, then redirected to login, and after logging in, added to group.
7. Given Alice has multiple groups, then all listed with names and member counts.
8. Zero JS console errors during group creation, listing, invite flows.
9. **Error path:** Given Alice submits create group with empty name, then validation error, no group created.

---

### Story 4 - Expense Creation (Equal Split)
**Priority:** P1
**Labels:** `mvp`, `story-4`

Users can record expenses within a group and split them equally among selected members.

**Dependencies:** Story 3

**Acceptance Criteria:**

1. Given Alice, Bob, Charlie are members of "Trip to Paris", when Alice clicks "Add Expense", then form has: description, amount, who paid, members to split among, date.
2. Given Alice enters "Dinner", $90.00, paid by Alice, split among all three, then expense appears: "Dinner -- $90.00 -- paid by Alice".
3. Given $90.00 split among three, then each share is $30.00.
4. Given $100.00 split among three, then shares are $33.34, $33.33, $33.33 (remainder distributed deterministically).
5. Given Alice adds an expense, then group detail shows it with description, amount, payer, date.
6. Zero JS console errors during expense creation.
7. **Error path:** Given amount "0" or negative, then validation error, no expense created.
8. **Error path:** Given no members selected for split, then validation error.

---

### Story 5 - Balance Calculation & Display
**Priority:** P1
**Labels:** `mvp`, `story-5`

Users can see who owes whom within a group, with debts simplified to minimize the number of payments needed.

**Dependencies:** Story 4

**Acceptance Criteria:**

1. Given Alice paid $90 split equally among Alice/Bob/Charlie, when Bob views group detail, then balances show "Bob owes Alice $30.00" and "Charlie owes Alice $30.00".
2. Given the same, when Alice views balances, then shown as owed $60.00 total.
3. Given Alice paid $90 and Bob paid $60 (both split three ways), then debts are simplified to minimum transfers.
4. Given a group with no expenses, then balances show "All settled up!".
5. **Cross-story:** Given Alice created group (Story 3) and added expense (Story 4), when Bob views group detail, then sees expense list AND calculated balance on same page.
6. Zero JS console errors.
7. **Error path:** Given only Alice in a group with an expense she paid, then no debts shown.

---

### Story 6 - Settlement Recording
**Priority:** P1
**Labels:** `mvp`, `story-6`

Users can record payments between members to settle debts, with the ability to undo recent settlements.

**Dependencies:** Story 5

**Acceptance Criteria:**

1. Given Bob owes Alice $30, when Bob clicks "Record Settlement", then form has: who paid, who received, amount, date.
2. Given Bob submits settlement (Bob->Alice $30), then balances update -- Bob no longer owes Alice.
3. Given Bob recorded a settlement < 24h ago, then "Delete" button visible.
4. Given Bob clicks "Delete", then settlement removed, balances revert (Bob owes Alice $30 again).
5. Given a settlement > 24h old, then no "Delete" button shown.
6. **Cross-story:** Given Alice added $90 expense (Story 4), Bob settled $30, Charlie hasn't, when Alice views balances, then Bob=$0, Charlie owes $30.
7. Zero JS console errors.
8. **Error path:** Given amount "0" or negative on settlement form, then validation error.

---

### Story 7 - Azure Infrastructure (Bicep)
**Priority:** P2
**Labels:** `mvp`, `story-7`

Infrastructure-as-Code templates provision all Azure resources needed to run SplitVibe in production.

**Dependencies:** Story 0 (can be worked in parallel with Stories 1-6)

**Acceptance Criteria:**

1. Given Bicep templates in `infra/`, when running `az deployment group validate`, then exit 0. *(Command strategy -- requires Azure CLI.)*
2. Resources defined: Container Apps Environment, Container App, PostgreSQL Flexible Server, Blob Storage, Container Registry, Key Vault, Log Analytics.
3. PostgreSQL configured with private VNet access.
4. Database password and OAuth credentials stored in Key Vault, referenced by Container App.
5. Given project root, when running `docker build -t splitvibe-test .`, then build succeeds. *(Command strategy.)*
6. Given Docker image, when running container with test env vars, then `/api/health` responds HTTP 200 within 30s. *(Command strategy.)*

**Note:** Criterion 1 requires Azure CLI. If not available -> BLOCKED. Ask user. Criteria 2-4 must be validated via `az bicep build` or the validate command, NOT by reading source.

---

### Story 8 - CI/CD Pipeline
**Priority:** P2
**Labels:** `mvp`, `story-8`

Automated build, test, and deployment pipeline so every PR is validated and merges to main are deployed to Azure.

**Dependencies:** Story 0 (can be worked in parallel with Stories 1-6)

**Acceptance Criteria:**

1. Given `.github/workflows/ci.yml`, when PR is opened, then GitHub Actions runs: fmt, clippy, test, build.
2. Given CI workflow, when any step fails, then PR check shows failed. *(Verify via `gh pr checks`.)*
3. Given `.github/workflows/deploy.yml`, then it triggers on push to `main`.
4. Given deploy workflow, then it includes: Docker build, push to ACR, Container App revision update.
5. **Cross-story:** Given Docker build succeeds (Story 7) and CI passes, then app can be built, tested, and packaged end-to-end.
6. Given this PR, when running `gh pr checks` after CI completes, then all checks pass. *(Command strategy.)*

**Note:** Deploy workflow won't run until merged. Deployment verification -> BLOCKED if Azure not configured.

---

## Execution Order

```
Story 0 (scaffold)
  |
  +-- Story 1 (schema) --+-- Story 2 (auth) -- Story 3 (groups) -- Story 4 (expenses) -- Story 5 (balances) -- Story 6 (settlements)
  |                       |
  +-- Story 7 (infra)     |  (Stories 1 & 2 can be parallelized after Story 0)
  |                       |
  +-- Story 8 (CI/CD) ----+
```

## Future Stories (Post-MVP)

- Percentage and weighted split modes
- Multi-currency expenses with FX rate capture
- Receipt/attachment uploads via Azure Blob Storage
- PWA manifest and offline support
- Expense editing and deletion
- Member removal and group archiving
- Global dashboard with cross-group balances
- Apple OAuth provider
