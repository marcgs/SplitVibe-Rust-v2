use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        if let Some(response) = use_context::<leptos_actix::ResponseOptions>() {
            response.set_status(actix_web::http::StatusCode::NOT_FOUND);
        }
    }

    view! {
        <div class="container">
            <h1>"404 - Page Not Found"</h1>
            <p>"The page you are looking for does not exist."</p>
            <a href="/">"Go Home"</a>
        </div>
    }
}
