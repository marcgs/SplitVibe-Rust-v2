use actix_session::Session;
use actix_web::{web, HttpResponse};
use splitvibe_core::models::MOCK_USERS;

use super::session::{self, SessionUser};

#[derive(serde::Deserialize)]
pub struct MockLoginForm {
    pub user_id: String,
}

pub async fn mock_login(
    session: Session,
    pool: web::Data<sqlx::PgPool>,
    form: web::Form<MockLoginForm>,
) -> HttpResponse {
    let mock_auth_enabled = std::env::var("MOCK_AUTH_ENABLED").unwrap_or_default() == "true";

    if !mock_auth_enabled {
        return HttpResponse::Forbidden().body("Mock login is disabled");
    }

    let mock_user = MOCK_USERS.iter().find(|u| u.id == form.user_id);
    let Some(mock_user) = mock_user else {
        return HttpResponse::BadRequest().body("Unknown mock user");
    };

    // Upsert user in database
    let user_id = mock_user.id.to_string();
    let display_name = mock_user.display_name.to_string();
    let avatar_url = mock_user.avatar_url.to_string();

    let result = sqlx::query(
        r#"INSERT INTO users (id, provider, provider_id, display_name, avatar_url)
           VALUES ($1, 'mock', $1, $2, $3)
           ON CONFLICT (id) DO UPDATE SET display_name = $2, avatar_url = $3, updated_at = now()"#,
    )
    .bind(&user_id)
    .bind(&display_name)
    .bind(&avatar_url)
    .execute(pool.get_ref())
    .await;

    if let Err(e) = result {
        tracing::error!("Failed to upsert mock user: {}", e);
        return HttpResponse::InternalServerError().body("Database error");
    }

    // Clear any existing session and set new user
    session::clear_session(&session);

    let session_user = SessionUser {
        id: user_id,
        display_name,
        avatar_url: Some(avatar_url),
    };

    if let Err(e) = session::set_session_user(&session, &session_user) {
        tracing::error!("Failed to set session: {}", e);
        return HttpResponse::InternalServerError().body("Session error");
    }

    HttpResponse::SeeOther()
        .insert_header(("Location", "/groups"))
        .finish()
}

pub async fn logout(session: Session) -> HttpResponse {
    session::clear_session(&session);
    HttpResponse::SeeOther()
        .insert_header(("Location", "/auth/login"))
        .finish()
}

pub async fn google_login() -> HttpResponse {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();

    if client_id == "not-configured" || client_id.is_empty() {
        return HttpResponse::SeeOther()
            .insert_header(("Location", "/auth/login?error=google_not_configured"))
            .finish();
    }

    // TODO: Implement actual Google OAuth flow
    HttpResponse::InternalServerError().body("Google OAuth not yet implemented")
}

pub async fn get_current_user(session: Session) -> HttpResponse {
    match session::get_session_user(&session) {
        Some(user) => HttpResponse::Ok().json(user),
        None => HttpResponse::Unauthorized().finish(),
    }
}
