use rust_decimal_macros::dec;
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
    insert_mock_user(pool, "charlie-003", "Charlie").await;

    queries::create_group(
        pool,
        "grp-1",
        "Trip to Paris",
        "alice-001",
        "tok-1",
        "mem-1",
    )
    .await
    .unwrap();
    queries::add_group_member(pool, "mem-2", "grp-1", "bob-002")
        .await
        .unwrap();
    queries::add_group_member(pool, "mem-3", "grp-1", "charlie-003")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_create_expense_returns_expense() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    let splits = vec![
        ("sp-1".into(), "alice-001".into(), dec!(30.00)),
        ("sp-2".into(), "bob-002".into(), dec!(30.00)),
        ("sp-3".into(), "charlie-003".into(), dec!(30.00)),
    ];

    let expense = queries::create_expense(
        &pool,
        "exp-1",
        "grp-1",
        "Dinner",
        dec!(90.00),
        "alice-001",
        "alice-001",
        chrono::Local::now().date_naive(),
        "pay-1",
        &splits,
    )
    .await
    .expect("Failed to create expense");

    assert_eq!(expense.title, "Dinner");
    assert_eq!(expense.amount, dec!(90.00));
    assert_eq!(expense.group_id, "grp-1");
}

#[tokio::test]
async fn test_list_expenses_for_group() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    let splits = vec![
        ("sp-1".into(), "alice-001".into(), dec!(30.00)),
        ("sp-2".into(), "bob-002".into(), dec!(30.00)),
        ("sp-3".into(), "charlie-003".into(), dec!(30.00)),
    ];

    queries::create_expense(
        &pool,
        "exp-1",
        "grp-1",
        "Dinner",
        dec!(90.00),
        "alice-001",
        "alice-001",
        chrono::Local::now().date_naive(),
        "pay-1",
        &splits,
    )
    .await
    .unwrap();

    let expenses = queries::list_expenses_for_group(&pool, "grp-1")
        .await
        .expect("Failed to list expenses");

    assert_eq!(expenses.len(), 1);
    assert_eq!(expenses[0].title, "Dinner");
    assert_eq!(expenses[0].amount, dec!(90.00));
    assert_eq!(expenses[0].payer_name, "Alice");
}

#[tokio::test]
async fn test_list_expenses_excludes_deleted() {
    let pool = setup_pool().await;
    setup_group_with_members(&pool).await;

    let splits = vec![("sp-1".into(), "alice-001".into(), dec!(50.00))];

    queries::create_expense(
        &pool,
        "exp-1",
        "grp-1",
        "Lunch",
        dec!(50.00),
        "alice-001",
        "alice-001",
        chrono::Local::now().date_naive(),
        "pay-1",
        &splits,
    )
    .await
    .unwrap();

    // Mark as deleted
    sqlx::query("UPDATE expenses SET deleted = true WHERE id = 'exp-1'")
        .execute(&pool)
        .await
        .unwrap();

    let expenses = queries::list_expenses_for_group(&pool, "grp-1")
        .await
        .unwrap();
    assert_eq!(expenses.len(), 0);
}
