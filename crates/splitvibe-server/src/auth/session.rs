use actix_session::Session;
use serde::{Deserialize, Serialize};

const USER_SESSION_KEY: &str = "user";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

pub fn get_session_user(session: &Session) -> Option<SessionUser> {
    session
        .get::<String>(USER_SESSION_KEY)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
}

pub fn set_session_user(
    session: &Session,
    user: &SessionUser,
) -> Result<(), actix_session::SessionInsertError> {
    let json = serde_json::to_string(user).expect("SessionUser should serialize");
    session.insert(USER_SESSION_KEY, json)
}

pub fn clear_session(session: &Session) {
    session.purge();
}
