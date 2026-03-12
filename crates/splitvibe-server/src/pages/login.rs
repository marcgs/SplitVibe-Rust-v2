use leptos::prelude::*;
use splitvibe_core::models::MOCK_USERS;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Sign In"</h1>
            <div class="login-options">
                <h2>"Development Login"</h2>
                {MOCK_USERS
                    .iter()
                    .map(|user| {
                        let name = user.display_name;
                        let avatar = user.avatar_url;
                        view! {
                            <button class="mock-login-btn">
                                <img src=avatar alt=name width="32" height="32"/>
                                {format!("Login as {}", name)}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="login-options">
                <h2>"Production Login"</h2>
                <button class="google-login-btn">"Sign in with Google"</button>
            </div>
        </div>
    }
}
