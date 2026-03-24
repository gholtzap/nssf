use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::doc;
use futures::TryStreamExt;
use serde::Deserialize;

use crate::errors::{AppError, AppResult};
use crate::types::AppState;
use crate::types::db::SlicePolicy;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyQuery {
    pub sst: Option<u8>,
    pub sd: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn list_policies(
    State(state): State<AppState>,
    Query(query): Query<PolicyQuery>,
) -> AppResult<Json<Vec<SlicePolicy>>> {
    let collection = state.db.collection::<SlicePolicy>("policies");

    let mut filter = doc! {};
    if let Some(sst) = query.sst {
        filter.insert("snssai.sst", sst as i32);
    }
    if let Some(sd) = &query.sd {
        filter.insert("snssai.sd", sd);
    }
    if let Some(mcc) = &query.mcc {
        filter.insert("plmnId.mcc", mcc);
    }
    if let Some(mnc) = &query.mnc {
        filter.insert("plmnId.mnc", mnc);
    }
    if let Some(enabled) = query.enabled {
        filter.insert("enabled", enabled);
    }

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query policies: {}", e))
    })?;

    let policies: Vec<SlicePolicy> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect policies: {}", e))
    })?;

    Ok(Json(policies))
}

pub async fn get_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
) -> AppResult<Json<SlicePolicy>> {
    let collection = state.db.collection::<SlicePolicy>("policies");

    let policy = collection
        .find_one(doc! { "policyId": &policy_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query policy: {}", e))
        })?;

    match policy {
        Some(p) => Ok(Json(p)),
        None => Err(AppError::NotFound("Policy not found".to_string())),
    }
}

pub async fn create_policy(
    State(state): State<AppState>,
    Json(policy): Json<SlicePolicy>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<SlicePolicy>("policies");

    let existing = collection
        .find_one(doc! { "policyId": &policy.policy_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing policy: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Policy already exists with this policy ID".to_string(),
        ));
    }

    collection.insert_one(policy).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create policy: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Policy created successfully" })),
    ))
}

pub async fn update_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
    Json(updates): Json<SlicePolicy>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<SlicePolicy>("policies");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(doc! { "policyId": &policy_id }, doc! { "$set": update_doc })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update policy: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound("Policy not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "Policy updated successfully" }),
    ))
}

pub async fn delete_policy(
    State(state): State<AppState>,
    Path(policy_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<SlicePolicy>("policies");

    let result = collection
        .delete_one(doc! { "policyId": &policy_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete policy: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound("Policy not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "Policy deleted successfully" }),
    ))
}
