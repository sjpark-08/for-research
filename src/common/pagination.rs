use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_size")]
    pub size: u32
}

fn default_page() -> u32 { 0 }
fn default_size() -> u32 { 10 }

#[derive(Debug, Serialize, ToSchema)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub size: u32,
    pub total_items: i64,
    pub total_pages: u32,
}