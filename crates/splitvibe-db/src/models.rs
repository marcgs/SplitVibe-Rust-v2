use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub provider: String,
    pub provider_id: Option<String>,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub preferred_currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub data: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub base_currency: String,
    pub invite_token: String,
    pub created_by: String,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupMember {
    pub id: String,
    pub group_id: String,
    pub user_id: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "split_mode", rename_all = "lowercase")]
pub enum SplitMode {
    Equal,
    Percentage,
    Shares,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Expense {
    pub id: String,
    pub group_id: String,
    pub title: String,
    pub amount: Decimal,
    pub currency: String,
    pub split_mode: SplitMode,
    pub expense_date: NaiveDate,
    pub fx_rate: Option<Decimal>,
    pub created_by: String,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExpensePayer {
    pub id: String,
    pub expense_id: String,
    pub user_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExpenseSplit {
    pub id: String,
    pub expense_id: String,
    pub user_id: String,
    pub amount: Decimal,
    pub share_value: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Settlement {
    pub id: String,
    pub group_id: String,
    pub payer_id: String,
    pub payee_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
}
