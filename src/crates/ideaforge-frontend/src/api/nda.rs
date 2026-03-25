use super::client;
use super::types::{NdaStatusResponse, NdaTemplateResponse, SignNdaRequest};

pub async fn get_nda_template(idea_id: &str) -> Result<NdaTemplateResponse, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/nda/template")).await
}

pub async fn sign_nda(idea_id: &str, signer_name: &str) -> Result<NdaStatusResponse, client::ApiError> {
    let req = SignNdaRequest {
        signer_name: signer_name.to_string(),
    };
    client::post(&format!("/api/v1/ideas/{idea_id}/nda/sign"), &req).await
}

pub async fn get_nda_status(idea_id: &str) -> Result<NdaStatusResponse, client::ApiError> {
    client::get(&format!("/api/v1/ideas/{idea_id}/nda/status")).await
}
