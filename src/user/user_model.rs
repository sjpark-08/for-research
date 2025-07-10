use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct UserCreate {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct UserUpdate {
    pub id: i64,
    pub email: String,
    pub username: String,
}

#[derive(Serialize, Debug, Clone, ToResponse, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
        }
    }
}