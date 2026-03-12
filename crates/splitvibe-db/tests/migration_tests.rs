use sqlx::PgPool;

async fn setup_pool() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Clean slate: drop all tables and types, then re-run migrations
    sqlx::raw_sql(
        "DO $$ DECLARE r RECORD;
         BEGIN
           FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
             EXECUTE 'DROP TABLE IF EXISTS public.' || quote_ident(r.tablename) || ' CASCADE';
           END LOOP;
           DROP TYPE IF EXISTS split_mode CASCADE;
         END $$;",
    )
    .execute(&pool)
    .await
    .expect("Failed to clean database");

    splitvibe_db::run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn migrations_apply_successfully() {
    let _pool = setup_pool().await;
    // If we get here, migrations applied without error
}

#[tokio::test]
async fn required_tables_exist() {
    let pool = setup_pool().await;

    let expected_tables = [
        "users",
        "groups",
        "group_members",
        "expenses",
        "expense_payers",
        "expense_splits",
        "settlements",
        "sessions",
    ];

    for table in &expected_tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = $1
            )",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        assert!(exists, "Table '{}' should exist", table);
    }
}

#[tokio::test]
async fn expenses_amount_is_decimal_14_4() {
    let pool = setup_pool().await;

    let (data_type, precision, scale): (String, Option<i32>, Option<i32>) = sqlx::query_as(
        "SELECT data_type, numeric_precision, numeric_scale
         FROM information_schema.columns
         WHERE table_name = 'expenses' AND column_name = 'amount'",
    )
    .fetch_one(&pool)
    .await
    .expect("Should find expenses.amount column");

    assert_eq!(data_type, "numeric");
    assert_eq!(precision, Some(14));
    assert_eq!(scale, Some(4));
}

#[tokio::test]
async fn foreign_keys_exist() {
    let pool = setup_pool().await;

    let fk_checks = [
        ("expenses", "group_id", "groups", "id"),
        ("group_members", "user_id", "users", "id"),
        ("settlements", "group_id", "groups", "id"),
    ];

    for (table, column, ref_table, ref_column) in &fk_checks {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.key_column_usage kcu
                JOIN information_schema.referential_constraints rc
                  ON kcu.constraint_name = rc.constraint_name
                JOIN information_schema.key_column_usage rcu
                  ON rc.unique_constraint_name = rcu.constraint_name
                WHERE kcu.table_name = $1
                  AND kcu.column_name = $2
                  AND rcu.table_name = $3
                  AND rcu.column_name = $4
            )",
        )
        .bind(table)
        .bind(column)
        .bind(ref_table)
        .bind(ref_column)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);

        assert!(
            exists,
            "FK {}.{} → {}.{} should exist",
            table, column, ref_table, ref_column
        );
    }
}

#[tokio::test]
async fn migrations_are_idempotent() {
    let pool = setup_pool().await;

    // Running migrations again should succeed (idempotent)
    let result = splitvibe_db::run_migrations(&pool).await;
    assert!(result.is_ok(), "Re-running migrations should succeed");
}

#[tokio::test]
async fn tables_are_queryable() {
    let pool = setup_pool().await;

    // Verify each table can be queried (SELECT)
    let tables = [
        "users",
        "groups",
        "group_members",
        "expenses",
        "expense_payers",
        "expense_splits",
        "settlements",
        "sessions",
    ];

    for table in &tables {
        let query = format!("SELECT COUNT(*) as count FROM {}", table);
        let count: (i64,) = sqlx::query_as(&query)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|e| panic!("Failed to query table '{}': {}", table, e));

        assert_eq!(count.0, 0, "Table '{}' should be empty initially", table);
    }
}
