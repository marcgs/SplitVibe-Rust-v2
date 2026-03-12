#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use splitvibe_server::app::{shell, App};

    dotenvy::dotenv().ok();

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    tracing::info!("Starting SplitVibe server at http://{}", addr);

    let routes = generate_route_list(App);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        App::new()
            .app_data(web::Data::new(leptos_options.clone()))
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
