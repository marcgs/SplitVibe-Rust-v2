-- SplitVibe initial schema

-- Custom types
CREATE TYPE split_mode AS ENUM ('equal', 'percentage', 'shares');

-- Users
CREATE TABLE users (
    id              TEXT PRIMARY KEY,
    provider        TEXT NOT NULL DEFAULT 'google',
    provider_id     TEXT,
    email           TEXT,
    display_name    TEXT NOT NULL,
    avatar_url      TEXT,
    preferred_currency TEXT NOT NULL DEFAULT 'USD',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_id)
);

-- Sessions (server-side, for actix-session)
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    data        JSONB NOT NULL DEFAULT '{}',
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- Groups
CREATE TABLE groups (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    cover_image_url TEXT,
    base_currency   TEXT NOT NULL DEFAULT 'USD',
    invite_token    TEXT NOT NULL UNIQUE,
    created_by      TEXT NOT NULL REFERENCES users(id),
    archived        BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Group members
CREATE TABLE group_members (
    id          TEXT PRIMARY KEY,
    group_id    TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (group_id, user_id)
);
CREATE INDEX idx_group_members_group_id ON group_members(group_id);
CREATE INDEX idx_group_members_user_id ON group_members(user_id);

-- Expenses
CREATE TABLE expenses (
    id          TEXT PRIMARY KEY,
    group_id    TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    amount      DECIMAL(14,4) NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'USD',
    split_mode  split_mode NOT NULL DEFAULT 'equal',
    expense_date DATE NOT NULL DEFAULT CURRENT_DATE,
    fx_rate     DECIMAL(14,8),
    created_by  TEXT NOT NULL REFERENCES users(id),
    deleted     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_expenses_group_id ON expenses(group_id);

-- Expense payers (who paid)
CREATE TABLE expense_payers (
    id          TEXT PRIMARY KEY,
    expense_id  TEXT NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES users(id),
    amount      DECIMAL(14,4) NOT NULL,
    UNIQUE (expense_id, user_id)
);
CREATE INDEX idx_expense_payers_expense_id ON expense_payers(expense_id);

-- Expense splits (who owes)
CREATE TABLE expense_splits (
    id          TEXT PRIMARY KEY,
    expense_id  TEXT NOT NULL REFERENCES expenses(id) ON DELETE CASCADE,
    user_id     TEXT NOT NULL REFERENCES users(id),
    amount      DECIMAL(14,4) NOT NULL,
    share_value DECIMAL(14,4),
    UNIQUE (expense_id, user_id)
);
CREATE INDEX idx_expense_splits_expense_id ON expense_splits(expense_id);

-- Settlements
CREATE TABLE settlements (
    id          TEXT PRIMARY KEY,
    group_id    TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    payer_id    TEXT NOT NULL REFERENCES users(id),
    payee_id    TEXT NOT NULL REFERENCES users(id),
    amount      DECIMAL(14,4) NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'USD',
    deleted     BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_settlements_group_id ON settlements(group_id);
