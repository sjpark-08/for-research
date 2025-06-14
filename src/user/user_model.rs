use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct UserUpdate {
    pub id: i64,
    pub name: String,
    pub email: String
}

#[derive(Serialize, Debug, Clone, ToResponse)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            name: user.name,
            email: user.email
        }
    }
}