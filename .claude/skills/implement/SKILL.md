---
name: implement
description: >
  Implements a GitHub issue (story) end-to-end following the project's TDD
  workflow, opens a pull request, and then invokes the validate-pr skill to
  verify that all acceptance criteria are met.
---

# Implement Story

Implement a GitHub issue, open a PR, and validate it.

## Input

A GitHub issue number or URL from the **marcgs/SplitVibe-Rust-v2** repository.

## Steps

### 1. Read and understand the issue

- Fetch the **GitHub Issue** by number from the `marcgs/SplitVibe-Rust-v2` repository.
- Extract the following sections verbatim:
  - **Description** -- what the story is about.
  - **Implementation Details** -- key technical decisions, affected files,
    non-obvious notes.
  - **Acceptance Criteria** -- the checklist of conditions that must be true.
- If the issue has no acceptance criteria, report an error and stop.

### 2. Study the codebase

- Read the project instructions:
  - `docs/spec.md` -- product requirements and domain rules.
  - `docs/tech.md` -- architecture, auth flow, deployment, env vars.
  - `docs/backlog.md` -- story dependencies and context.
  - `crates/splitvibe-db/migrations/` -- current database migrations.
- Explore the current codebase to understand existing patterns, file
  structure, and conventions (Leptos components, server functions,
  test structure, Actix-web handlers).
- Identify which files need to be created or modified.

### 3. Create a feature branch

- Create a new Git branch from the default branch (`main`):
  ```
  git checkout main && git pull origin main
  git checkout -b copilot/issue-<number>-<short-slug>
  ```
- The `<short-slug>` should be a kebab-case summary of the issue title
  (e.g., `copilot/issue-42-percentage-split-mode`).

### 4. Plan the implementation

Before writing any code, produce a brief implementation plan:

1. List every acceptance criterion and the files affected.
2. Identify the order of implementation -- respect dependencies
   (e.g., migrations before models before server functions before UI).
3. For each piece, note which test file(s) will cover it.

Share the plan for the user to review before proceeding.

### 5. Implement using TDD

**Follow the project's TDD workflow strictly:**

For each logical unit of work:

1. **Write a failing test first** that describes the desired behavior.
   - Unit tests go alongside the code in the same module.
   - Integration tests go in `tests/` directories within each crate.
2. **Run the test** to confirm it fails:
   ```bash
   cargo test -p <crate> -- <test_name>
   ```
3. **Implement the minimal code** to make the test pass.
4. **Run the test again** to confirm it passes.
5. **Refactor** while keeping tests green.
6. **Repeat** for the next unit of work.

#### Implementation guidelines

- **Use `cargo clippy` and `cargo fmt`** -- zero warnings policy.
- **Follow existing patterns** in the codebase for consistency.
- For SQLx migrations, create numbered migration files:
  ```bash
  sqlx migrate add <descriptive-name>
  ```
- For new Leptos pages, follow existing patterns in `crates/splitvibe-server/src/pages/`.
- For new components, follow patterns in `crates/splitvibe-server/src/components/`.
- For new server functions, follow Leptos server function patterns.
- Use `rust_decimal::Decimal` for monetary amounts.
- Use `cuid2` for ID generation.

#### `.env.example` consistency

- When adding or changing environment variables, **always update `.env.example`** in the same commit.
- `SESSION_SECRET` in `.env.example` must be >= 64 bytes.
- Smoke test: `cp .env.example .env && docker compose up -d && cargo leptos serve` must work without manual edits.

#### Mock users

Development mode must have **three mock users**: Alice, Bob, and Charlie. Each with a distinct `id`, `display_name`, and `avatar_url`. The login page must show three buttons: "Login as Alice", "Login as Bob", "Login as Charlie".

#### README

For scaffold stories (Story 0), create or update `README.md` with: project description, prerequisites, setup instructions, how to run tests, and how to access the app.

### 6. Quality checks

After all implementation is complete, run the full quality suite:

```bash
cargo fmt --check             # Must pass
cargo clippy -- -D warnings   # Must pass with zero warnings
cargo test                    # All tests must pass
```

Fix any issues before proceeding. Do not skip this step.

### 7. Commit and push

- Stage all changes and create a well-structured commit (or multiple
  commits for logical units):
  ```bash
  git add -A
  git commit -m "feat: <short description>

  <longer description if needed>

  Closes #<issue-number>"
  ```
- Push the branch:
  ```bash
  git push -u origin copilot/issue-<number>-<short-slug>
  ```

### 8. Open a pull request

Create a pull request on GitHub with:

- **Title:** A concise summary of the change.
- **Body:** Include:
  - A brief description of what was implemented and why.
  - A link to the issue: `Closes #<issue-number>`.
  - A summary of the changes (files added/modified, key decisions).
  - Any notes for reviewers.
- **Base branch:** `main`
- **Head branch:** `copilot/issue-<number>-<short-slug>`

### 9. Validate the PR

After the PR is created, invoke the **validate-pr** skill to verify that
all acceptance criteria are met:

```
/validate-pr PR #<pr-number>
```

The validate-pr skill will **post its validation report as a comment on the
PR** (see validate-pr Step 7). Ensure the full report table, summary, and
conclusion are visible in the PR timeline so reviewers can see the
validation status without re-running the skill. If a previous validation
comment already exists, it should be updated rather than duplicated.

- If validation passes -- report success to the user.
- If validation fails -- inspect the failures, fix the code, push
  updated commits, and re-run validation until all criteria pass or
  the issue is clearly identified. Each re-validation must update the
  existing PR comment with the latest results.
- If validation finds test coverage gaps -- add the missing tests,
  push, and re-validate.

### 10. Final report

```markdown
## Implementation Report -- Issue #<number>

### Story: <issue title>

| Phase | Status | Notes |
|-------|--------|-------|
| Issue read | DONE | <N> acceptance criteria found |
| Branch created | DONE | `copilot/issue-<N>-<slug>` |
| Tests written | DONE | <N> test files, <M> test cases |
| Implementation | DONE | <N> files created, <M> modified |
| Clippy | DONE | Zero warnings |
| Fmt | DONE | Clean |
| All tests pass | DONE | <N>/<N> passing |
| PR opened | DONE | PR #<pr-number> |
| Validation | DONE | All criteria verified |

### PR: #<pr-number> -- <pr-title>
### Validation: All acceptance criteria verified
```
