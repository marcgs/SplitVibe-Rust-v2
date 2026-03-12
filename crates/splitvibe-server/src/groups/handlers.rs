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

    let expenses =
        match splitvibe_db::queries::list_expenses_for_group(pool.get_ref(), &group_id).await {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Failed to list expenses: {}", e);
                return HttpResponse::InternalServerError().body("Database error");
            }
        };

    let expenses_html = if expenses.is_empty() {
        r#"<p class="empty-state">No expenses yet.</p>"#.to_string()
    } else {
        let items: Vec<String> = expenses
            .iter()
            .map(|e| {
                format!(
                    r#"<div class="expense-item">
                        <div class="expense-info">
                            <span class="expense-title">{title}</span>
                            <span class="expense-meta">{date} &middot; paid by {payer}</span>
                        </div>
                        <span class="expense-amount">${amount}</span>
                    </div>"#,
                    title = html_escape(&e.title),
                    amount = e.amount.round_dp(2),
                    payer = html_escape(&e.payer_name),
                    date = e.expense_date,
                )
            })
            .collect();
        format!(r#"<div class="expense-list">{}</div>"#, items.join("\n"))
    };

    // Fetch balance data and calculate debts
    let balances_html =
        match splitvibe_db::queries::get_expense_data_for_balances(pool.get_ref(), &group_id).await
        {
            Ok((payers, splits)) => {
                // Group by expense_id to create ExpenseEntry list
                let mut expense_map: std::collections::HashMap<
                    String,
                    splitvibe_core::balance::ExpenseEntry,
                > = std::collections::HashMap::new();
                for p in &payers {
                    expense_map.entry(p.expense_id.clone()).or_insert_with(|| {
                        splitvibe_core::balance::ExpenseEntry {
                            payer: p.user_id.clone(),
                            splits: Vec::new(),
                        }
                    });
                }
                for s in &splits {
                    if let Some(entry) = expense_map.get_mut(&s.expense_id) {
                        entry.splits.push((s.user_id.clone(), s.amount));
                    }
                }

                let entries: Vec<splitvibe_core::balance::ExpenseEntry> =
                    expense_map.into_values().collect();
                let debts = splitvibe_core::balance::calculate_debts(&entries);

                if debts.is_empty() {
                    r#"<p class="empty-state">All settled up!</p>"#.to_string()
                } else {
                    // Map user IDs to display names
                    let name_map: std::collections::HashMap<&str, &str> = members
                        .iter()
                        .map(|m| (m.user_id.as_str(), m.display_name.as_str()))
                        .collect();

                    let items: Vec<String> = debts
                        .iter()
                        .map(|d| {
                            let from_name =
                                name_map.get(d.from.as_str()).unwrap_or(&d.from.as_str());
                            let to_name = name_map.get(d.to.as_str()).unwrap_or(&d.to.as_str());
                            format!(
                                r#"<div class="balance-item">
                                <span class="balance-from">{from}</span>
                                <span class="balance-arrow">owes</span>
                                <span class="balance-to">{to}</span>
                                <span class="balance-amount">${amount}</span>
                            </div>"#,
                                from = html_escape(from_name),
                                to = html_escape(to_name),
                                amount = d.amount.round_dp(2),
                            )
                        })
                        .collect();
                    format!(r#"<div class="balance-list">{}</div>"#, items.join("\n"))
                }
            }
            Err(e) => {
                tracing::error!("Failed to get balance data: {}", e);
                r#"<p class="empty-state">Could not calculate balances.</p>"#.to_string()
            }
        };

    let invite_url = format!("/join/{}", html_escape(&group.invite_token));

    let content = format!(
        r#"<h1>{name}</h1>
        <div class="group-detail">
            <div class="group-actions">
                <a href="/groups/{group_id}/expenses/new" class="btn btn-primary">Add Expense</a>
                <button type="button" class="btn btn-secondary" id="copy-invite"
                    data-invite-url="{invite_url}"
                    onclick="navigator.clipboard.writeText(window.location.origin + this.dataset.inviteUrl).then(function(){{document.getElementById('copy-invite').textContent='Copied!';setTimeout(function(){{document.getElementById('copy-invite').textContent='Copy invite link'}},2000)}})">Copy invite link</button>
            </div>
            <h2>Expenses</h2>
            {expenses}
            <h2>Balances</h2>
            {balances}
            <h2>Members ({count})</h2>
            <div class="member-list">
                {members}
            </div>
            <a href="/groups" class="btn btn-secondary">Back to groups</a>
        </div>"#,
        name = html_escape(&group.name),
        group_id = html_escape(&group.id),
        invite_url = invite_url,
        expenses = expenses_html,
        balances = balances_html,
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

/// GET /groups/{id}/expenses/new — show the add expense form.
pub async fn expenses_new(
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

    let content = expense_form_html(&group.id, &group.name, &members, None, None);
    let navbar = navbar_html(&user);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page_html("Add Expense", &navbar, &content))
}

fn expense_form_html(
    group_id: &str,
    group_name: &str,
    members: &[splitvibe_db::queries::GroupMemberInfo],
    error: Option<&str>,
    form_values: Option<&CreateExpenseForm>,
) -> String {
    let payer_options: Vec<String> = members
        .iter()
        .map(|m| {
            let selected = form_values
                .map(|f| f.payer_id == m.user_id)
                .unwrap_or(false);
            format!(
                r#"<option value="{id}" {sel}>{name}</option>"#,
                id = html_escape(&m.user_id),
                name = html_escape(&m.display_name),
                sel = if selected { "selected" } else { "" },
            )
        })
        .collect();

    let member_checkboxes: Vec<String> = members
        .iter()
        .map(|m| {
            let checked = form_values
                .map(|f| f.split_members.contains(&m.user_id))
                .unwrap_or(true); // default: all checked
            format!(
                r#"<label class="checkbox-label">
                    <input type="checkbox" name="split_members" value="{id}" {checked}/>
                    {name}
                </label>"#,
                id = html_escape(&m.user_id),
                name = html_escape(&m.display_name),
                checked = if checked { "checked" } else { "" },
            )
        })
        .collect();

    let error_html = error
        .map(|msg| format!(r#"<div class="error-message">{}</div>"#, html_escape(msg)))
        .unwrap_or_default();

    let amount_val = form_values
        .map(|f| html_escape(&f.amount))
        .unwrap_or_default();
    let title_val = form_values
        .map(|f| html_escape(&f.title))
        .unwrap_or_default();
    let date_val = form_values
        .map(|f| f.date.clone())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());

    format!(
        r#"<h1>Add Expense to {group_name}</h1>
        {error_html}
        <form method="post" action="/groups/{group_id}/expenses" class="form">
            <div class="form-group">
                <label for="title">Description</label>
                <input type="text" id="title" name="title" required placeholder="e.g. Dinner" class="form-input" value="{title_val}"/>
            </div>
            <div class="form-group">
                <label for="amount">Amount ($)</label>
                <input type="number" id="amount" name="amount" required step="0.01" min="0.01" placeholder="0.00" class="form-input" value="{amount_val}"/>
            </div>
            <div class="form-group">
                <label for="payer_id">Paid by</label>
                <select id="payer_id" name="payer_id" class="form-input">
                    {payer_options}
                </select>
            </div>
            <div class="form-group">
                <label>Split among</label>
                <div class="checkbox-group">
                    {member_checkboxes}
                </div>
            </div>
            <div class="form-group">
                <label for="date">Date</label>
                <input type="date" id="date" name="date" class="form-input" value="{date_val}"/>
            </div>
            <button type="submit" class="btn btn-primary">Add Expense</button>
            <a href="/groups/{group_id}" class="btn btn-secondary">Cancel</a>
        </form>"#,
        group_name = html_escape(group_name),
        group_id = html_escape(group_id),
        payer_options = payer_options.join("\n"),
        member_checkboxes = member_checkboxes.join("\n"),
    )
}

#[derive(Clone)]
pub struct CreateExpenseForm {
    pub title: String,
    pub amount: String,
    pub payer_id: String,
    pub split_members: Vec<String>,
    pub date: String,
}

fn parse_expense_form(body: &[u8]) -> CreateExpenseForm {
    let body_str = String::from_utf8_lossy(body);
    let mut title = String::new();
    let mut amount = String::new();
    let mut payer_id = String::new();
    let mut split_members = Vec::new();
    let mut date = String::new();

    for pair in body_str.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("").replace('+', " ");
        let value = urlencoding::decode(&value)
            .unwrap_or(std::borrow::Cow::Borrowed(""))
            .to_string();
        match key {
            "title" => title = value,
            "amount" => amount = value,
            "payer_id" => payer_id = value,
            "split_members" => {
                if !value.is_empty() {
                    split_members.push(value);
                }
            }
            "date" => date = value,
            _ => {}
        }
    }

    CreateExpenseForm {
        title,
        amount,
        payer_id,
        split_members,
        date,
    }
}

/// POST /groups/{id}/expenses — create a new expense.
pub async fn expenses_create(
    session: Session,
    pool: web::Data<sqlx::PgPool>,
    path: web::Path<String>,
    body: web::Bytes,
) -> HttpResponse {
    // Parse form manually to handle repeated split_members[] fields
    let form = parse_expense_form(&body);
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

    // Validate
    let show_error = |msg: &str| {
        let content = expense_form_html(&group.id, &group.name, &members, Some(msg), Some(&form));
        let navbar = navbar_html(&user);
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(page_html("Add Expense", &navbar, &content))
    };

    if let Err(msg) = splitvibe_core::validation::validate_expense_title(&form.title) {
        return show_error(msg);
    }

    let amount = match splitvibe_core::validation::validate_expense_amount(&form.amount) {
        Ok(a) => a,
        Err(msg) => return show_error(msg),
    };

    if let Err(msg) = splitvibe_core::validation::validate_split_members(&form.split_members) {
        return show_error(msg);
    }

    // Build participant names for the split calculation
    let participant_names: Vec<(String, String)> = form
        .split_members
        .iter()
        .filter_map(|uid| {
            members
                .iter()
                .find(|m| m.user_id == *uid)
                .map(|m| (uid.clone(), m.display_name.clone()))
        })
        .collect();

    let names_only: Vec<String> = participant_names.iter().map(|(_, n)| n.clone()).collect();

    let split_result = splitvibe_core::split::split_equal(amount, names_only);

    // Map split results back to user IDs
    let splits: Vec<(String, String, rust_decimal::Decimal)> = split_result
        .shares
        .iter()
        .map(|(name, share_amount)| {
            let user_id = participant_names
                .iter()
                .find(|(_, n)| n == name)
                .map(|(uid, _)| uid.clone())
                .unwrap_or_default();
            (cuid2::create_id(), user_id, *share_amount)
        })
        .collect();

    let expense_date = form
        .date
        .parse::<chrono::NaiveDate>()
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    let expense_id = cuid2::create_id();
    let payer_record_id = cuid2::create_id();

    match splitvibe_db::queries::create_expense(
        pool.get_ref(),
        &expense_id,
        &group.id,
        form.title.trim(),
        amount,
        &form.payer_id,
        &user.id,
        expense_date,
        &payer_record_id,
        &splits,
    )
    .await
    {
        Ok(_) => HttpResponse::SeeOther()
            .insert_header(("Location", format!("/groups/{}", group.id)))
            .finish(),
        Err(e) => {
            tracing::error!("Failed to create expense: {}", e);
            HttpResponse::InternalServerError().body("Database error")
        }
    }
}
