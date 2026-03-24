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
use crate::types::db::NsagConfiguration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsagQuery {
    pub sst: Option<u8>,
    pub sd: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn list_nsags(
    State(state): State<AppState>,
    Query(query): Query<NsagQuery>,
) -> AppResult<Json<Vec<NsagConfiguration>>> {
    let collection = state.db.collection::<NsagConfiguration>("nsag_configurations");

    let mut filter = doc! {};
    if let Some(sst) = query.sst {
        filter.insert("snssaiList.sst", sst as i32);
    }
    if let Some(sd) = &query.sd {
        filter.insert("snssaiList.sd", sd);
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
        AppError::InternalServerError(format!("Failed to query NSAG configurations: {}", e))
    })?;

    let nsags: Vec<NsagConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect NSAG configurations: {}", e))
    })?;

    Ok(Json(nsags))
}

pub async fn get_nsag(
    State(state): State<AppState>,
    Path(nsag_id): Path<u32>,
) -> AppResult<Json<NsagConfiguration>> {
    let collection = state.db.collection::<NsagConfiguration>("nsag_configurations");

    let nsag = collection
        .find_one(doc! { "nsagId": nsag_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query NSAG configuration: {}", e))
        })?;

    match nsag {
        Some(n) => Ok(Json(n)),
        None => Err(AppError::NotFound(
            "NSAG configuration not found".to_string(),
        )),
    }
}

pub async fn create_nsag(
    State(state): State<AppState>,
    Json(nsag): Json<NsagConfiguration>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<NsagConfiguration>("nsag_configurations");

    let existing = collection
        .find_one(doc! { "nsagId": nsag.nsag_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing NSAG: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "NSAG configuration already exists for this NSAG ID".to_string(),
        ));
    }

    collection.insert_one(nsag).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create NSAG configuration: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "NSAG configuration created successfully" })),
    ))
}

pub async fn update_nsag(
    State(state): State<AppState>,
    Path(nsag_id): Path<u32>,
    Json(updates): Json<NsagConfiguration>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NsagConfiguration>("nsag_configurations");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(doc! { "nsagId": nsag_id }, doc! { "$set": update_doc })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update NSAG configuration: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(
            "NSAG configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSAG configuration updated successfully" }),
    ))
}

pub async fn delete_nsag(
    State(state): State<AppState>,
    Path(nsag_id): Path<u32>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NsagConfiguration>("nsag_configurations");

    let result = collection
        .delete_one(doc! { "nsagId": nsag_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete NSAG configuration: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(
            "NSAG configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSAG configuration deleted successfully" }),
    ))
}
