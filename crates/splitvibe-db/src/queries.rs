use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::models::{Expense, Group, GroupMember};

/// A group with its member count for listing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct GroupWithMemberCount {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub base_currency: String,
    pub member_count: i64,
}

/// A group member with display info.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct GroupMemberInfo {
    pub user_id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

/// Create a group and add the creator as the first member.
pub async fn create_group(
    pool: &PgPool,
    group_id: &str,
    name: &str,
    created_by: &str,
    invite_token: &str,
    member_id: &str,
) -> Result<Group, sqlx::Error> {
    let group = sqlx::query_as::<_, Group>(
        r#"INSERT INTO groups (id, name, created_by, invite_token)
           VALUES ($1, $2, $3, $4)
           RETURNING *"#,
    )
    .bind(group_id)
    .bind(name)
    .bind(created_by)
    .bind(invite_token)
    .fetch_one(pool)
    .await?;

    sqlx::query("INSERT INTO group_members (id, group_id, user_id) VALUES ($1, $2, $3)")
        .bind(member_id)
        .bind(group_id)
        .bind(created_by)
        .execute(pool)
        .await?;

    Ok(group)
}

/// List all groups for a user with member counts.
pub async fn list_groups_for_user(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<GroupWithMemberCount>, sqlx::Error> {
    sqlx::query_as::<_, GroupWithMemberCount>(
        r#"SELECT g.id, g.name, g.description, g.base_currency,
                  (SELECT COUNT(*) FROM group_members gm2 WHERE gm2.group_id = g.id) AS member_count
           FROM groups g
           JOIN group_members gm ON g.id = gm.group_id
           WHERE gm.user_id = $1 AND g.archived = false
           ORDER BY g.created_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get a group by ID.
pub async fn get_group_by_id(pool: &PgPool, group_id: &str) -> Result<Option<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE id = $1")
        .bind(group_id)
        .fetch_optional(pool)
        .await
}

/// Get a group by invite token.
pub async fn get_group_by_invite_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Group>, sqlx::Error> {
    sqlx::query_as::<_, Group>("SELECT * FROM groups WHERE invite_token = $1")
        .bind(token)
        .fetch_optional(pool)
        .await
}

/// Get members of a group with user display info.
pub async fn get_group_members(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<GroupMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, GroupMemberInfo>(
        r#"SELECT gm.user_id, u.display_name, u.avatar_url, gm.joined_at
           FROM group_members gm
           JOIN users u ON gm.user_id = u.id
           WHERE gm.group_id = $1
           ORDER BY gm.joined_at ASC"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

/// Check if a user is already a member of a group.
pub async fn is_group_member(
    pool: &PgPool,
    group_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(row > 0)
}

/// Add a user to a group. Returns the new GroupMember or None if already a member.
pub async fn add_group_member(
    pool: &PgPool,
    member_id: &str,
    group_id: &str,
    user_id: &str,
) -> Result<Option<GroupMember>, sqlx::Error> {
    let result = sqlx::query_as::<_, GroupMember>(
        r#"INSERT INTO group_members (id, group_id, user_id)
           VALUES ($1, $2, $3)
           ON CONFLICT (group_id, user_id) DO NOTHING
           RETURNING *"#,
    )
    .bind(member_id)
    .bind(group_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(result)
}

/// Expense summary for display in group detail.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct ExpenseSummary {
    pub id: String,
    pub title: String,
    pub amount: Decimal,
    pub currency: String,
    pub expense_date: chrono::NaiveDate,
    pub payer_name: String,
}

/// Create an expense with payer and split records.
#[allow(clippy::too_many_arguments)]
pub async fn create_expense(
    pool: &PgPool,
    expense_id: &str,
    group_id: &str,
    title: &str,
    amount: Decimal,
    payer_user_id: &str,
    created_by: &str,
    expense_date: chrono::NaiveDate,
    payer_record_id: &str,
    splits: &[(String, String, Decimal)], // (split_id, user_id, amount)
) -> Result<Expense, sqlx::Error> {
    let expense = sqlx::query_as::<_, Expense>(
        r#"INSERT INTO expenses (id, group_id, title, amount, created_by, expense_date)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING *"#,
    )
    .bind(expense_id)
    .bind(group_id)
    .bind(title)
    .bind(amount)
    .bind(created_by)
    .bind(expense_date)
    .fetch_one(pool)
    .await?;

    // Insert payer record
    sqlx::query(
        "INSERT INTO expense_payers (id, expense_id, user_id, amount) VALUES ($1, $2, $3, $4)",
    )
    .bind(payer_record_id)
    .bind(expense_id)
    .bind(payer_user_id)
    .bind(amount)
    .execute(pool)
    .await?;

    // Insert split records
    for (split_id, user_id, split_amount) in splits {
        sqlx::query(
            "INSERT INTO expense_splits (id, expense_id, user_id, amount) VALUES ($1, $2, $3, $4)",
        )
        .bind(split_id)
        .bind(expense_id)
        .bind(user_id)
        .bind(split_amount)
        .execute(pool)
        .await?;
    }

    Ok(expense)
}

/// List expenses for a group with payer display name.
pub async fn list_expenses_for_group(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<ExpenseSummary>, sqlx::Error> {
    sqlx::query_as::<_, ExpenseSummary>(
        r#"SELECT e.id, e.title, e.amount, e.currency, e.expense_date,
                  u.display_name AS payer_name
           FROM expenses e
           JOIN expense_payers ep ON e.id = ep.expense_id
           JOIN users u ON ep.user_id = u.id
           WHERE e.group_id = $1 AND e.deleted = false
           ORDER BY e.expense_date DESC, e.created_at DESC"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

/// Raw payer record for balance calculation.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExpensePayerRecord {
    pub expense_id: String,
    pub user_id: String,
    pub amount: Decimal,
}

/// Raw split record for balance calculation.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExpenseSplitRecord {
    pub expense_id: String,
    pub user_id: String,
    pub amount: Decimal,
}

/// Fetch all payer and split records for non-deleted expenses in a group.
/// Returns (payers, splits) for use with the balance algorithm.
pub async fn get_expense_data_for_balances(
    pool: &PgPool,
    group_id: &str,
) -> Result<(Vec<ExpensePayerRecord>, Vec<ExpenseSplitRecord>), sqlx::Error> {
    let payers = sqlx::query_as::<_, ExpensePayerRecord>(
        r#"SELECT ep.expense_id, ep.user_id, ep.amount
           FROM expense_payers ep
           JOIN expenses e ON ep.expense_id = e.id
           WHERE e.group_id = $1 AND e.deleted = false"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    let splits = sqlx::query_as::<_, ExpenseSplitRecord>(
        r#"SELECT es.expense_id, es.user_id, es.amount
           FROM expense_splits es
           JOIN expenses e ON es.expense_id = e.id
           WHERE e.group_id = $1 AND e.deleted = false"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;

    Ok((payers, splits))
}

/// Settlement with display names for the payer and payee.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SettlementInfo {
    pub id: String,
    pub payer_name: String,
    pub payee_name: String,
    pub payer_id: String,
    pub payee_id: String,
    pub amount: Decimal,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub can_delete: bool,
}

/// Create a settlement record.
pub async fn create_settlement(
    pool: &PgPool,
    id: &str,
    group_id: &str,
    payer_id: &str,
    payee_id: &str,
    amount: Decimal,
) -> Result<crate::models::Settlement, sqlx::Error> {
    sqlx::query_as::<_, crate::models::Settlement>(
        "INSERT INTO settlements (id, group_id, payer_id, payee_id, amount) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(id)
    .bind(group_id)
    .bind(payer_id)
    .bind(payee_id)
    .bind(amount)
    .fetch_one(pool)
    .await
}

/// List active settlements for a group with display names and delete eligibility.
pub async fn list_settlements_for_group(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<SettlementInfo>, sqlx::Error> {
    sqlx::query_as::<_, SettlementInfo>(
        r#"SELECT s.id, u1.display_name AS payer_name, u2.display_name AS payee_name,
                  s.payer_id, s.payee_id, s.amount, s.created_at,
                  (s.created_at > now() - interval '24 hours') AS can_delete
           FROM settlements s
           JOIN users u1 ON s.payer_id = u1.id
           JOIN users u2 ON s.payee_id = u2.id
           WHERE s.group_id = $1 AND s.deleted = false
           ORDER BY s.created_at DESC"#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
}

/// Soft-delete a settlement (set deleted=true, deleted_at=now()).
pub async fn delete_settlement(
    pool: &PgPool,
    settlement_id: &str,
    group_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE settlements SET deleted = true, deleted_at = now() WHERE id = $1 AND group_id = $2 AND deleted = false AND created_at > now() - interval '24 hours'",
    )
    .bind(settlement_id)
    .bind(group_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Fetch active settlements for balance calculation.
pub async fn get_settlements_for_balances(
    pool: &PgPool,
    group_id: &str,
) -> Result<Vec<(String, String, Decimal)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Decimal)>(
        "SELECT payer_id, payee_id, amount FROM settlements WHERE group_id = $1 AND deleted = false",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
