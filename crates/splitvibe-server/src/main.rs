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
            // Protected groups page (SSR with auth check)
            .route("/groups", web::get().to(groups_page))
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

/// Server-side rendered groups page with authentication check.
#[cfg(feature = "ssr")]
async fn groups_page(session: actix_session::Session) -> actix_web::HttpResponse {
    use splitvibe_server::auth::session::get_session_user;

    let user = get_session_user(&session);

    match user {
        Some(user) => {
            let avatar_html = user
                .avatar_url
                .as_deref()
                .map(|url| {
                    format!(
                        r#"<img src="{}" alt="avatar" class="navbar-avatar" width="32" height="32"/>"#,
                        url
                    )
                })
                .unwrap_or_default();

            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>SplitVibe - Groups</title>
    <link id="leptos" rel="stylesheet" href="/pkg/splitvibe.css">
</head>
<body>
    <nav class="navbar">
        <div class="navbar-brand"><a href="/">SplitVibe</a></div>
        <div class="navbar-user">
            {avatar}
            <span class="navbar-username">{name}</span>
            <form method="post" action="/auth/logout" class="navbar-logout">
                <button type="submit">Sign out</button>
            </form>
        </div>
    </nav>
    <main>
        <div class="container">
            <h1>My Groups</h1>
            <p>No groups yet. Create one to get started!</p>
        </div>
    </main>
</body>
</html>"#,
                avatar = avatar_html,
                name = user.display_name,
            );
            actix_web::HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(html)
        }
        None => actix_web::HttpResponse::SeeOther()
            .insert_header(("Location", "/auth/login"))
            .finish(),
    }
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Server requires the `ssr` feature. Use `cargo leptos serve` to run.
}
