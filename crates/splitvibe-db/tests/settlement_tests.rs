use rust_decimal_macros::dec;
use serial_test::serial;
use splitvibe_db::queries;
use sqlx::PgPool;

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    splitvibe_db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

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

async fn setup_group_with_members(pool: &PgPool) {
    insert_mock_user(pool, "alice-001", "Alice").await;
    insert_mock_user(pool, "bob-002", "Bob").await;

    queries::create_group(pool, "grp-1", "Trip", "alice-001", "tok-1", "mem-1")
        .await
        .unwrap();
    queries::add_group_member(pool, "mem-2", "grp-1", "bob-002")
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
async fn test_create_settlement() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    let settlement =
        queries::create_settlement(&pool, "stl-1", "grp-1", "bob-002", "alice-001", dec!(30.00))
            .await
            .expect("Failed to create settlement");

    assert_eq!(settlement.payer_id, "bob-002");
    assert_eq!(settlement.payee_id, "alice-001");
    assert_eq!(settlement.amount, dec!(30.00));
    assert!(!settlement.deleted);
}

#[tokio::test]
#[serial]
async fn test_list_settlements_for_group() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    queries::create_settlement(&pool, "stl-1", "grp-1", "bob-002", "alice-001", dec!(30.00))
        .await
        .unwrap();

    let settlements = queries::list_settlements_for_group(&pool, "grp-1")
        .await
        .expect("Failed to list settlements");

    assert_eq!(settlements.len(), 1);
    assert_eq!(settlements[0].payer_name, "Bob");
    assert_eq!(settlements[0].payee_name, "Alice");
    assert_eq!(settlements[0].amount, dec!(30.00));
    assert!(settlements[0].can_delete);
}

#[tokio::test]
#[serial]
async fn test_delete_settlement_within_24h() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    queries::create_settlement(&pool, "stl-1", "grp-1", "bob-002", "alice-001", dec!(30.00))
        .await
        .unwrap();

    let deleted = queries::delete_settlement(&pool, "stl-1", "grp-1")
        .await
        .expect("Failed to delete settlement");
    assert!(deleted);

    let settlements = queries::list_settlements_for_group(&pool, "grp-1")
        .await
        .unwrap();
    assert_eq!(settlements.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_get_settlements_for_balances() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    queries::create_settlement(&pool, "stl-1", "grp-1", "bob-002", "alice-001", dec!(30.00))
        .await
        .unwrap();

    let rows = queries::get_settlements_for_balances(&pool, "grp-1")
        .await
        .expect("Failed to get settlements for balances");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].0, "bob-002");
    assert_eq!(rows[0].1, "alice-001");
    assert_eq!(rows[0].2, dec!(30.00));
}

#[tokio::test]
#[serial]
async fn test_deleted_settlement_excluded_from_balances() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    queries::create_settlement(&pool, "stl-1", "grp-1", "bob-002", "alice-001", dec!(30.00))
        .await
        .unwrap();

    queries::delete_settlement(&pool, "stl-1", "grp-1")
        .await
        .unwrap();

    let rows = queries::get_settlements_for_balances(&pool, "grp-1")
        .await
        .unwrap();
    assert_eq!(rows.len(), 0);
}
