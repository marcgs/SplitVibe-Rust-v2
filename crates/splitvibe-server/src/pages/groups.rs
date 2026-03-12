use leptos::prelude::*;

use crate::components::navbar::Navbar;

#[component]
pub fn GroupsPage(user_name: String, avatar_url: Option<String>) -> impl IntoView {
    view! {
        <Navbar user_name=user_name avatar_url=avatar_url/>
        <div class="container">
            <h1>"My Groups"</h1>
            <p>"No groups yet. Create one to get started!"</p>
        </div>
    }
}
