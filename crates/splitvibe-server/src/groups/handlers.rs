use actix_session::Session;
use actix_web::{web, HttpResponse};

use crate::auth::session::{get_session_user, SessionUser};

fn navbar_html(user: &SessionUser) -> String {
    let avatar_html = user
        .avatar_url
        .as_deref()
        .map(|url| {
            format!(
                r#"<img src="{}" alt="avatar" class="navbar-avatar" width="32" height="32"/>"#,
                html_escape(url)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<nav class="navbar">
        <div class="navbar-brand"><a href="/">SplitVibe</a></div>
        <div class="navbar-user">
            {avatar}
            <span class="navbar-username">{name}</span>
            <form method="post" action="/auth/logout" class="navbar-logout">
                <button type="submit">Sign out</button>
            </form>
        </div>
    </nav>"#,
        avatar = avatar_html,
        name = html_escape(&user.display_name),
    )
}

fn page_html(title: &str, navbar: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>SplitVibe - {title}</title>
    <link id="leptos" rel="stylesheet" href="/pkg/splitvibe.css">
</head>
<body>
    {navbar}
    <main>
        <div class="container">
            {content}
        </div>
    </main>
</body>
</html>"#,
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn require_auth(session: &Session) -> Result<SessionUser, HttpResponse> {
    get_session_user(session).ok_or_else(|| {
        HttpResponse::SeeOther()
            .insert_header(("Location", "/auth/login"))
            .finish()
    })
}

/// GET /groups — list all groups for the logged-in user.
pub async fn groups_list(session: Session, pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    let user = match require_auth(&session) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let groups = match splitvibe_db::queries::list_groups_for_user(pool.get_ref(), &user.id).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("Failed to list groups: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let groups_html = if groups.is_empty() {
        r#"<p>No groups yet. Create one to get started!</p>"#.to_string()
    } else {
        let items: Vec<String> = groups
            .iter()
            .map(|g| {
                format!(
                    r#"<a href="/groups/{id}" class="group-card">
                        <span class="group-card-name">{name}</span>
                        <span class="group-card-members">{count} {label}</span>
                    </a>"#,
                    id = html_escape(&g.id),
                    name = html_escape(&g.name),
                    count = g.member_count,
                    label = if g.member_count == 1 {
                        "member"
                    } else {
                        "members"
                    },
                )
            })
            .collect();
        format!(r#"<div class="group-list">{}</div>"#, items.join("\n"))
    };

    let content = format!(
        r#"<h1>My Groups</h1>
        <a href="/groups/new" class="btn btn-primary">Create Group</a>
        {groups_html}"#,
    );

    let navbar = navbar_html(&user);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_html("Groups", &navbar, &content))
}

/// GET /groups/new — show the create group form.
pub async fn groups_new(session: Session) -> HttpResponse {
    let user = match require_auth(&session) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let content = r#"<h1>Create Group</h1>
        <form method="post" action="/groups" class="form">
            <div class="form-group">
                <label for="name">Group Name</label>
                <input type="text" id="name" name="name" required placeholder="e.g. Trip to Paris" class="form-input"/>
            </div>
            <button type="submit" class="btn btn-primary">Create</button>
            <a href="/groups" class="btn btn-secondary">Cancel</a>
        </form>"#;

    let navbar = navbar_html(&user);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_html("Create Group", &navbar, content))
}

#[derive(serde::Deserialize)]
pub struct CreateGroupForm {
    pub name: String,
}

/// POST /groups — create a new group.
pub async fn groups_create(
    session: Session,
    pool: web::Data<sqlx::PgPool>,
    form: web::Form<CreateGroupForm>,
) -> HttpResponse {
    let user = match require_auth(&session) {
        Ok(u) => u,
        Err(r) => return r,
    };

    if let Err(msg) = splitvibe_core::validation::validate_group_name(&form.name) {
        let content = format!(
            r#"<h1>Create Group</h1>
            <div class="error-message">{msg}</div>
            <form method="post" action="/groups" class="form">
                <div class="form-group">
                    <label for="name">Group Name</label>
                    <input type="text" id="name" name="name" required placeholder="e.g. Trip to Paris" class="form-input" value="{value}"/>
                </div>
                <button type="submit" class="btn btn-primary">Create</button>
                <a href="/groups" class="btn btn-secondary">Cancel</a>
            </form>"#,
            msg = html_escape(msg),
            value = html_escape(&form.name),
        );
        let navbar = navbar_html(&user);
        return HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(page_html("Create Group", &navbar, &content));
    }

    let group_id = cuid2::create_id();
    let invite_token = cuid2::create_id();
    let member_id = cuid2::create_id();

    match splitvibe_db::queries::create_group(
        pool.get_ref(),
        &group_id,
        form.name.trim(),
        &user.id,
        &invite_token,
        &member_id,
    )
    .await
    {
        Ok(_) => HttpResponse::SeeOther()
            .insert_header(("Location", format!("/groups/{}", group_id)))
            .finish(),
        Err(e) => {
            tracing::error!("Failed to create group: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}

/// GET /groups/{id} — group detail page.
pub async fn groups_detail(
    session: Session,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let user = match require_auth(&session) {
        Ok(u) => u,
        Err(r) => return r,
    };

    let group_id = path.into_inner();

    let group = match splitvibe_db::queries::get_group_by_id(pool.get_ref(), &group_id).await {
        Ok(Some(g)) => g,
        Ok(None) => return HttpResponse::NotFound().body("Group not found"),
        Err(e) => {
            tracing::error!("Failed to get group: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let members = match splitvibe_db::queries::get_group_members(pool.get_ref(), &group_id).await {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to get group members: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    let members_html: Vec<String> = members
        .iter()
        .map(|m| {
            let avatar = m
                .avatar_url
                .as_deref()
                .map(|url| {
                    format!(
                        r#"<img src="{}" alt="avatar" class="member-avatar" width="32" height="32"/>"#,
                        html_escape(url)
                    )
                })
                .unwrap_or_default();
            format!(
                r#"<div class="member-item">{avatar}<span>{name}</span></div>"#,
                name = html_escape(&m.display_name),
            )
        })
        .collect();

    let invite_url = format!("/join/{}", html_escape(&group.invite_token));

    let content = format!(
        r#"<h1>{name}</h1>
        <div class="group-detail">
            <div class="group-invite">
                <button type="button" class="btn btn-secondary" id="copy-invite"
                    data-invite-url="{invite_url}"
                    onclick="navigator.clipboard.writeText(window.location.origin + this.dataset.inviteUrl).then(function(){{document.getElementById('copy-invite').textContent='Copied!';setTimeout(function(){{document.getElementById('copy-invite').textContent='Copy invite link'}},2000)}})">Copy invite link</button>
            </div>
            <h2>Members ({count})</h2>
            <div class="member-list">
                {members}
            </div>
            <a href="/groups" class="btn btn-secondary">Back to groups</a>
        </div>"#,
        name = html_escape(&group.name),
        invite_url = invite_url,
        count = members.len(),
        members = members_html.join("\n"),
    );

    let navbar = navbar_html(&user);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_html(&group.name, &navbar, &content))
}

/// GET /join/{token} — join a group via invite link.
pub async fn join_group(
    session: Session,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let token = path.into_inner();

    // Look up the group by invite token
    let group = match splitvibe_db::queries::get_group_by_invite_token(pool.get_ref(), &token).await
    {
        Ok(Some(g)) => g,
        Ok(None) => return HttpResponse::NotFound().body("Invalid invite link"),
        Err(e) => {
            tracing::error!("Failed to look up invite token: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    // Check if user is logged in
    let user = match get_session_user(&session) {
        Some(u) => u,
        None => {
            // Store the join URL in session so we can redirect after login
            let _ = session.insert("redirect_after_login", format!("/join/{}", token));
            return HttpResponse::SeeOther()
                .insert_header(("Location", "/auth/login"))
                .finish();
        }
    };

    // Try to add user to group
    let member_id = cuid2::create_id();
    match splitvibe_db::queries::add_group_member(pool.get_ref(), &member_id, &group.id, &user.id)
        .await
    {
        Ok(Some(_)) => {
            // Successfully added
            HttpResponse::SeeOther()
                .insert_header(("Location", format!("/groups/{}", group.id)))
                .finish()
        }
        Ok(None) => {
            // Already a member — show message then redirect
            let navbar = navbar_html(&user);
            let content = format!(
                r#"<h1>Already a Member</h1>
                <p>You are already a member of <strong>{name}</strong>.</p>
                <a href="/groups/{id}" class="btn btn-primary">Go to group</a>"#,
                name = html_escape(&group.name),
                id = html_escape(&group.id),
            );
            HttpResponse::Ok()
                .content_type("text/html; charset=utf-8")
                .body(page_html("Already a Member", &navbar, &content))
        }
        Err(e) => {
            tracing::error!("Failed to add group member: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}
