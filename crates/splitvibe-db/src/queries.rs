use sqlx::PgPool;

use crate::models::{Group, GroupMember};

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
