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
use crate::types::db::NssrgConfiguration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssrgQuery {
    pub sst: Option<u8>,
    pub sd: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn list_nssrgs(
    State(state): State<AppState>,
    Query(query): Query<NssrgQuery>,
) -> AppResult<Json<Vec<NssrgConfiguration>>> {
    let collection = state.db.collection::<NssrgConfiguration>("nssrg_configurations");

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
        AppError::InternalServerError(format!("Failed to query NSSRG configurations: {}", e))
    })?;

    let nssrgs: Vec<NssrgConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect NSSRG configurations: {}", e))
    })?;

    Ok(Json(nssrgs))
}

pub async fn get_nssrg(
    State(state): State<AppState>,
    Path(nssrg_id): Path<String>,
) -> AppResult<Json<NssrgConfiguration>> {
    let collection = state.db.collection::<NssrgConfiguration>("nssrg_configurations");

    let nssrg = collection
        .find_one(doc! { "nssrgId": &nssrg_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query NSSRG configuration: {}", e))
        })?;

    match nssrg {
        Some(n) => Ok(Json(n)),
        None => Err(AppError::NotFound(
            "NSSRG configuration not found".to_string(),
        )),
    }
}

pub async fn create_nssrg(
    State(state): State<AppState>,
    Json(nssrg): Json<NssrgConfiguration>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<NssrgConfiguration>("nssrg_configurations");

    let existing = collection
        .find_one(doc! { "nssrgId": &nssrg.nssrg_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing NSSRG: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "NSSRG configuration already exists for this NSSRG ID".to_string(),
        ));
    }

    collection.insert_one(nssrg).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create NSSRG configuration: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "NSSRG configuration created successfully" })),
    ))
}

pub async fn update_nssrg(
    State(state): State<AppState>,
    Path(nssrg_id): Path<String>,
    Json(updates): Json<NssrgConfiguration>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NssrgConfiguration>("nssrg_configurations");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(doc! { "nssrgId": &nssrg_id }, doc! { "$set": update_doc })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update NSSRG configuration: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(
            "NSSRG configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSSRG configuration updated successfully" }),
    ))
}

pub async fn delete_nssrg(
    State(state): State<AppState>,
    Path(nssrg_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NssrgConfiguration>("nssrg_configurations");

    let result = collection
        .delete_one(doc! { "nssrgId": &nssrg_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete NSSRG configuration: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(
            "NSSRG configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSSRG configuration deleted successfully" }),
    ))
}
