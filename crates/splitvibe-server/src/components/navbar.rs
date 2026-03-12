use leptos::prelude::*;

#[component]
pub fn Navbar(user_name: String, avatar_url: Option<String>) -> impl IntoView {
    view! {
        <nav class="navbar">
            <div class="navbar-brand">
                <a href="/">"SplitVibe"</a>
            </div>
            <div class="navbar-user">
                {avatar_url
                    .map(|url| {
                        view! { <img src=url alt="avatar" class="navbar-avatar" width="32" height="32"/> }
                    })}
                <span class="navbar-username">{user_name}</span>
                <form method="post" action="/auth/logout" class="navbar-logout">
                    <button type="submit">"Sign out"</button>
                </form>
            </div>
        </nav>
    }
}
