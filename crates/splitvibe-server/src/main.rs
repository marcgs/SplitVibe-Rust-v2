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
