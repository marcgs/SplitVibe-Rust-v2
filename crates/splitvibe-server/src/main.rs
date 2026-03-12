#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_session::{
        config::PersistentSession, storage::CookieSessionStore, SessionMiddleware,
    };
    use actix_web::{cookie::Key, *};
    use leptos::prelude::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use splitvibe_server::app::{shell, App};
    use splitvibe_server::auth;
    use splitvibe_server::groups;

    dotenvy::dotenv().ok();

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    // Database pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    splitvibe_db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    // Session secret
    let session_secret = std::env::var("SESSION_SECRET").expect("SESSION_SECRET must be set");
    assert!(
        session_secret.len() >= 64,
        "SESSION_SECRET must be at least 64 bytes"
    );
    let secret_key = Key::from(session_secret.as_bytes());

    tracing::info!("Starting SplitVibe server at http://{}", addr);

    let routes = generate_route_list(App);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        App::new()
            .app_data(web::Data::new(leptos_options.clone()))
            .app_data(web::Data::new(pool.clone()))
            // Session middleware
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .session_lifecycle(
                        PersistentSession::default()
                            .session_ttl(actix_web::cookie::time::Duration::days(7)),
                    )
                    .build(),
            )
            // Auth routes (before Leptos routes)
            .route(
                "/auth/mock-login",
                web::post().to(auth::handlers::mock_login),
            )
            .route("/auth/logout", web::post().to(auth::handlers::logout))
            .route("/auth/google", web::get().to(auth::handlers::google_login))
            .route(
                "/api/auth/me",
                web::get().to(auth::handlers::get_current_user),
            )
            // Group routes (SSR with auth check)
            .route("/groups", web::get().to(groups::handlers::groups_list))
            .route("/groups/new", web::get().to(groups::handlers::groups_new))
            .route("/groups", web::post().to(groups::handlers::groups_create))
            .route(
                "/groups/{id}",
                web::get().to(groups::handlers::groups_detail),
            )
            .route("/join/{token}", web::get().to(groups::handlers::join_group))
            // Leptos routes
            .leptos_routes(routes.clone(), {
                let options = leptos_options.clone();
                move || shell(options.clone())
            })
            .service(Files::new("/", &site_root))
            .default_service(web::to(|| async {
                HttpResponse::NotFound()
                    .content_type("text/html; charset=utf-8")
                    .body(
                        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>SplitVibe - Not Found</title>
    <link id="leptos" rel="stylesheet" href="/pkg/splitvibe.css">
</head>
<body>
    <main>
        <div class="container">
            <h1>404 - Page Not Found</h1>
            <p>The page you are looking for does not exist.</p>
            <a href="/">Go Home</a>
        </div>
    </main>
</body>
</html>"#,
                    )
            }))
            .wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Server requires the `ssr` feature. Use `cargo leptos serve` to run.
}
