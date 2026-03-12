use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Welcome to SplitVibe"</h1>
            <p>"Track shared expenses with friends and family."</p>
            <a href="/auth/login">"Get Started"</a>
        </div>
    }
}
