use super::client;
use super::types::CategoryResponse;

pub async fn list_categories() -> Result<Vec<CategoryResponse>, client::ApiError> {
    client::get("/api/v1/categories").await
}
