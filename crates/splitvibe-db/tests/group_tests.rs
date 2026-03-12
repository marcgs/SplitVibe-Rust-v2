use splitvibe_db::queries;
use sqlx::PgPool;

/// Each test gets a unique prefix to avoid conflicts when running in parallel.
async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Ensure migrations are applied (idempotent)
    splitvibe_db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    // Clean data tables (order matters for FK constraints)
    sqlx::raw_sql(
        "TRUNCATE group_members, expense_splits, expense_payers, expenses, settlements, groups, sessions, users CASCADE;",
    )
    .execute(&pool)
    .await
    .expect("Failed to truncate tables");

    pool
}

async fn insert_mock_user(pool: &PgPool, id: &str, name: &str) {
    sqlx::query(
        "INSERT INTO users (id, provider, provider_id, display_name, avatar_url) VALUES ($1, 'mock', $1, $2, 'https://example.com/avatar.png')"
    )
    .bind(id)
    .bind(name)
    .execute(pool)
    .await
    .expect("Failed to insert mock user");
}

#[tokio::test]
async fn test_create_group_returns_group_with_correct_fields() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;

    let group = queries::create_group(
        &pool,
        "grp-1",
        "Trip to Paris",
        "alice-001",
        "tok-1",
        "mem-1",
    )
    .await
    .expect("Failed to create group");

    assert_eq!(group.id, "grp-1");
    assert_eq!(group.name, "Trip to Paris");
    assert_eq!(group.created_by, "alice-001");
    assert_eq!(group.invite_token, "tok-1");
    assert_eq!(group.base_currency, "USD");
    assert!(!group.archived);
}

#[tokio::test]
async fn test_create_group_adds_creator_as_member() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;

    queries::create_group(
        &pool,
        "grp-1",
        "Trip to Paris",
        "alice-001",
        "tok-1",
        "mem-1",
    )
    .await
    .expect("Failed to create group");

    let is_member = queries::is_group_member(&pool, "grp-1", "alice-001")
        .await
        .expect("Failed to check membership");
    assert!(is_member);
}

#[tokio::test]
async fn test_list_groups_for_user_returns_groups_with_member_count() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;
    insert_mock_user(&pool, "bob-002", "Bob").await;

    queries::create_group(
        &pool,
        "grp-1",
        "Trip to Paris",
        "alice-001",
        "tok-1",
        "mem-1",
    )
    .await
    .unwrap();
    queries::add_group_member(&pool, "mem-2", "grp-1", "bob-002")
        .await
        .unwrap();

    let groups = queries::list_groups_for_user(&pool, "alice-001")
        .await
        .expect("Failed to list groups");

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].name, "Trip to Paris");
    assert_eq!(groups[0].member_count, 2);
}

#[tokio::test]
async fn test_get_group_by_invite_token() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;

    queries::create_group(
        &pool,
        "grp-1",
        "Trip to Paris",
        "alice-001",
        "tok-abc",
        "mem-1",
    )
    .await
    .unwrap();

    let group = queries::get_group_by_invite_token(&pool, "tok-abc")
        .await
        .expect("Failed to get group by token");

    assert!(group.is_some());
    assert_eq!(group.unwrap().name, "Trip to Paris");

    let none = queries::get_group_by_invite_token(&pool, "nonexistent")
        .await
        .expect("Failed to query");
    assert!(none.is_none());
}

#[tokio::test]
async fn test_add_group_member_prevents_duplicates() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;
    insert_mock_user(&pool, "bob-002", "Bob").await;

    queries::create_group(&pool, "grp-1", "Trip", "alice-001", "tok-1", "mem-1")
        .await
        .unwrap();

    // First add succeeds
    let result = queries::add_group_member(&pool, "mem-2", "grp-1", "bob-002")
        .await
        .expect("Failed to add member");
    assert!(result.is_some());

    // Second add returns None (already a member)
    let result = queries::add_group_member(&pool, "mem-3", "grp-1", "bob-002")
        .await
        .expect("Failed to add member");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_group_members_returns_user_info() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;
    insert_mock_user(&pool, "bob-002", "Bob").await;

    queries::create_group(&pool, "grp-1", "Trip", "alice-001", "tok-1", "mem-1")
        .await
        .unwrap();
    queries::add_group_member(&pool, "mem-2", "grp-1", "bob-002")
        .await
        .unwrap();

    let members = queries::get_group_members(&pool, "grp-1")
        .await
        .expect("Failed to get members");

    assert_eq!(members.len(), 2);
    assert_eq!(members[0].display_name, "Alice");
    assert_eq!(members[1].display_name, "Bob");
}

#[tokio::test]
async fn test_list_groups_excludes_non_member_groups() {
    let pool = setup_pool().await;
    insert_mock_user(&pool, "alice-001", "Alice").await;
    insert_mock_user(&pool, "bob-002", "Bob").await;

    queries::create_group(&pool, "grp-1", "Alice Group", "alice-001", "tok-1", "mem-1")
        .await
        .unwrap();
    queries::create_group(&pool, "grp-2", "Bob Group", "bob-002", "tok-2", "mem-2")
        .await
        .unwrap();

    let alice_groups = queries::list_groups_for_user(&pool, "alice-001")
        .await
        .unwrap();
    assert_eq!(alice_groups.len(), 1);
    assert_eq!(alice_groups[0].name, "Alice Group");
}
