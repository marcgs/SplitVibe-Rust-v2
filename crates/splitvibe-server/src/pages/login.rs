use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use splitvibe_core::models::MOCK_USERS;

#[component]
pub fn LoginPage() -> impl IntoView {
    let query = use_query_map();
    let error_message = move || {
        query.read().get("error").map(|e| match e.as_str() {
            "google_not_configured" => {
                "Google OAuth is not configured. Use mock login for development.".to_string()
            }
            _ => format!("Login error: {}", e),
        })
    };

    view! {
        <div class="container">
            <h1>"Sign In"</h1>
            {move || {
                error_message().map(|msg| {
                    view! { <div class="error-message">{msg}</div> }
                })
            }}
            <div class="login-options">
                <h2>"Development Login"</h2>
                {MOCK_USERS
                    .iter()
                    .map(|user| {
                        let name = user.display_name;
                        let avatar = user.avatar_url;
                        let user_id = user.id;
                        view! {
                            <form method="post" action="/auth/mock-login">
                                <input type="hidden" name="user_id" value=user_id/>
                                <button type="submit" class="mock-login-btn">
                                    <img src=avatar alt=name width="32" height="32"/>
                                    {format!("Login as {}", name)}
                                </button>
                            </form>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="login-options">
                <h2>"Production Login"</h2>
                <a href="/auth/google" rel="external" class="google-login-btn">"Sign in with Google"</a>
            </div>
        </div>
    }
}
