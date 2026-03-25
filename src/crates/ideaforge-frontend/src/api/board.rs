use super::client;
use super::types::{BoardResponse, CreateTaskRequest, TaskResponse, UpdateTaskStatusRequest};

pub async fn get_board(idea_id: &str) -> Result<BoardResponse, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/board")).await
}

pub async fn create_task(
    idea_id: &str,
    req: CreateTaskRequest,
) -> Result<TaskResponse, client::ApiError> {
    client::post(&format!("/api/v1/ideas/{idea_id}/tasks"), &req).await
}

pub async fn update_task_status(
    idea_id: &str,
    task_id: &str,
    status: &str,
) -> Result<TaskResponse, client::ApiError> {
    let req = UpdateTaskStatusRequest {
        status: status.to_string(),
    };
    client::put(
        &format!("/api/v1/ideas/{idea_id}/tasks/{task_id}/status"),
        &req,
    )
    .await
}

pub async fn delete_task(idea_id: &str, task_id: &str) -> Result<(), client::ApiError> {
    client::delete_req(&format!("/api/v1/ideas/{idea_id}/tasks/{task_id}")).await
}
