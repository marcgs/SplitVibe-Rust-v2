---
name: validate-pr
description: >
  Validates a PR's acceptance criteria from the user's perspective using
  browser-based E2E tests (Playwright MCP) and API calls. Reads the linked
  GitHub issue as the single source of truth for acceptance criteria.
---

# Validate PR Acceptance Criteria

Validate the acceptance criteria of the given PR.

## Input

A pull request number or URL from the **marcgs/SplitVibe-Rust-v2** repository.

## Validation Strategies

Available strategies:

- **Browser (E2E)** -- Criterion has **any** observable UI impact:
  pages, forms, navigation, lists, toasts, modals, visual feedback.
  Validate with Playwright MCP browser tools against
  `http://localhost:3000`.
- **API** -- Criterion involves HTTP endpoints (status codes,
  response shapes, auth guards). Validate with `curl`/`fetch`
  against `http://localhost:3000/api/...`.
- **Command** -- Criterion involves running a shell command and
  checking exit code or output (e.g., `docker build`, `sqlx migrate run`,
  `gh pr checks`, `az deployment group validate`, `cargo test`).

> **IMPORTANT: Code review is NEVER a validation strategy.** If a criterion
> cannot be validated via Browser, API, or Command, mark it **BLOCKED** with
> an explanation. Never mark PASS based on reading source code.

## Steps

### 1. Resolve the PR and linked GitHub issue

- Fetch the PR description from **GitHub**.
- Look for a `Closes #N`, `Fixes #N`, or `Resolves #N` reference in the PR
  body or in the linked issues sidebar.
- Open that **GitHub Issue** and extract the **Acceptance Criteria** checklist
  items verbatim.
- **GitHub is the single source of truth** -- do not fall back to local
  markdown files.
- If no linked issue or acceptance criteria can be found, report an error and
  stop.

### 2. Classify each acceptance criterion

For every acceptance criterion, assign **one or more** validation
strategies (Browser, API, Command). A single criterion can (and often
should) be validated through multiple complementary strategies.

> **Browser-first rule:** Always prefer Browser (E2E) validation.
> If a criterion has any user-visible aspect -- pages, forms, lists,
> feedback messages, navigation -- it **MUST** be validated through
> the browser. API validation is a complement for verifying response
> shapes, status codes, or auth guards -- never a substitute for
> browser testing of UI-facing criteria.

### 3. Check out the PR branch in a worktree

Use a **git worktree** so the user's current checkout is not disturbed.

- Extract the **head branch name** from the PR metadata fetched in step 1.
- Remove any stale worktree from a previous run:
  `git worktree remove ../splitvibe-validate --force 2>/dev/null`
- Create the worktree:
  `git worktree add ../splitvibe-validate <branch>`
- **`cd ../splitvibe-validate`** -- all subsequent commands run from this directory.

### 4. Start a clean dev environment

Always start from a **clean slate** to avoid stale state, old code, or
leftover data from previous runs.

1. **Tear down anything already running:**
   - Kill any process on port 3000
     (`lsof -ti:3000 | xargs kill 2>/dev/null`).
   - `docker compose down -v` -- stop and remove all containers and
     volumes.
2. **Start backend services:**
   - `docker compose up -d db storage` -- start Postgres and Azurite.
   - Wait for Postgres to be ready
     (`docker compose exec db pg_isready -U postgres`; retry if needed).
3. **Set up environment:**
   - `cp .env.example .env` -- use `.env.example` as-is, **no CLI env var overrides**.
4. **Apply migrations (if applicable):**
   - `sqlx migrate run` -- apply pending migrations to a fresh DB.
5. **Start the dev server** (async/detached so it keeps running):
   - `cargo leptos serve`
6. **Wait for the server to be ready:**
   - Poll `curl -sf http://localhost:3000` with retries (up to ~60 s).
   - If it still fails after retries, mark all criteria as
     BLOCKED and skip to step 7 (report).

Only proceed to validation once the dev server is reachable.

### 5. Validate each criterion

Execute each criterion using its assigned strategy:

#### Browser (E2E)

1. Navigate to the relevant page or flow.
2. Wait for the page to be fully loaded (key selector visible / network idle).
3. Interact as a real user -- fill forms, click buttons, follow redirects.
4. Assert expected outcomes: elements appear/disappear, messages shown, URL
   changes.
5. **MANDATORY: Capture a screenshot** via `browser_take_screenshot` after each
   key assertion. **No screenshot = BLOCKED.** Every Browser criterion must have
   at least one screenshot as evidence.
6. **MANDATORY: Check JS console errors** after every browser navigation using
   `browser_console_messages`. Any JS error (not warning) = **FAIL** for that
   criterion.

#### API

1. Construct the request (method, path, headers, body).
2. Execute via terminal: `curl -s -w "\n%{http_code}" -X METHOD URL`.
3. Assert: status code, response body shape, error messages, headers.
4. For authenticated endpoints, include the session token/cookie if available.

#### Command

1. Run the specified shell command.
2. Check exit code (0 = success unless stated otherwise).
3. Check output for expected content if specified in the criterion.
4. Record the command and its output as evidence.

#### External configuration pause

When a criterion requires external setup that may not be present (Google OAuth
credentials, Azure CLI login, production URLs), **STOP and ask the user**
whether the prerequisite is configured. Never mark PASS without actual
execution. Never assume external services are available.

### 6. On failure

- Record the failing assertion, evidence (screenshot or terminal output), and
  context (URL, command, test name).
- **Continue** with remaining criteria -- do not abort the entire run.

### 7. Report results

```markdown
## Validation Report -- PR #<number>

### Issue: <issue title> (#<issue number>)

| # | Criterion | Strategy | Result | Evidence |
|---|-----------|----------|--------|----------|
| 1 | <text> | Browser | PASS | Screenshot: <ref> |
| 2 | <text> | Browser + API | FAIL | Expected 201, got 500. Screenshot: <ref> |
| 3 | <text> | Command | PASS | `sqlx migrate run` exit 0 |
| 4 | <text> | Browser | BLOCKED | External config required -- asked user |

### Summary

- **Passed:** X
- **Failed:** Y
- **Blocked:** Z (prerequisites not met)
```

**Final conclusion -- use exactly one of:**

- **All acceptance criteria for this PR are verified.**
- **Some acceptance criteria failed validation. See details above.**

### 8. Post results to the PR

After producing the validation report, **post it as a comment on the
PR in GitHub**. Use the GitHub API (or the available GitHub MCP tools)
to add an issue comment on the pull request with the full report from
step 7.

- If a previous validation comment from this agent already exists on
  the PR, **update it** instead of creating a duplicate.
- The comment should contain the complete report table, summary, and
  conclusion so that reviewers can see the validation status directly
  in the PR timeline without re-running the agent.

### 9. Tear down the dev environment

After posting results, **always** clean up:

1. Stop the dev server (kill the process on port 3000).
2. `docker compose down -v` -- stop and remove all containers and volumes.
3. `cd` back to the original repository root.
4. `git worktree remove ../splitvibe-validate --force` -- remove the
   temporary worktree.
