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
use crate::types::db::SnssaiMapping;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnssaiMappingQuery {
    pub serving_sst: Option<u8>,
    pub serving_sd: Option<String>,
    pub home_sst: Option<u8>,
    pub home_sd: Option<String>,
    pub serving_mcc: Option<String>,
    pub serving_mnc: Option<String>,
    pub home_mcc: Option<String>,
    pub home_mnc: Option<String>,
}

pub async fn list_snssai_mappings(
    State(state): State<AppState>,
    Query(query): Query<SnssaiMappingQuery>,
) -> AppResult<Json<Vec<SnssaiMapping>>> {
    let collection = state.db.collection::<SnssaiMapping>("snssai_mappings");

    let mut filter = doc! {};
    if let Some(sst) = query.serving_sst {
        filter.insert("servingSnssai.sst", sst as i32);
    }
    if let Some(sd) = &query.serving_sd {
        filter.insert("servingSnssai.sd", sd);
    }
    if let Some(sst) = query.home_sst {
        filter.insert("homeSnssai.sst", sst as i32);
    }
    if let Some(sd) = &query.home_sd {
        filter.insert("homeSnssai.sd", sd);
    }
    if let Some(mcc) = &query.serving_mcc {
        filter.insert("servingPlmnId.mcc", mcc);
    }
    if let Some(mnc) = &query.serving_mnc {
        filter.insert("servingPlmnId.mnc", mnc);
    }
    if let Some(mcc) = &query.home_mcc {
        filter.insert("homePlmnId.mcc", mcc);
    }
    if let Some(mnc) = &query.home_mnc {
        filter.insert("homePlmnId.mnc", mnc);
    }

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query S-NSSAI mappings: {}", e))
    })?;

    let mappings: Vec<SnssaiMapping> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect S-NSSAI mappings: {}", e))
    })?;

    Ok(Json(mappings))
}

pub async fn get_snssai_mapping(
    State(state): State<AppState>,
    Path(mapping_id): Path<String>,
) -> AppResult<Json<SnssaiMapping>> {
    let collection = state.db.collection::<SnssaiMapping>("snssai_mappings");

    let mapping = collection
        .find_one(doc! { "mappingId": &mapping_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query S-NSSAI mapping: {}", e))
        })?;

    match mapping {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound("S-NSSAI mapping not found".to_string())),
    }
}

pub async fn create_snssai_mapping(
    State(state): State<AppState>,
    Json(mapping): Json<SnssaiMapping>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<SnssaiMapping>("snssai_mappings");

    let existing = collection
        .find_one(doc! { "mappingId": &mapping.mapping_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to check existing S-NSSAI mapping: {}",
                e
            ))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "S-NSSAI mapping already exists with this mapping ID".to_string(),
        ));
    }

    collection.insert_one(mapping).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create S-NSSAI mapping: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "S-NSSAI mapping created successfully" })),
    ))
}

pub async fn update_snssai_mapping(
    State(state): State<AppState>,
    Path(mapping_id): Path<String>,
    Json(updates): Json<SnssaiMapping>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<SnssaiMapping>("snssai_mappings");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(
            doc! { "mappingId": &mapping_id },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update S-NSSAI mapping: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound("S-NSSAI mapping not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "S-NSSAI mapping updated successfully" }),
    ))
}

pub async fn delete_snssai_mapping(
    State(state): State<AppState>,
    Path(mapping_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<SnssaiMapping>("snssai_mappings");

    let result = collection
        .delete_one(doc! { "mappingId": &mapping_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete S-NSSAI mapping: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound("S-NSSAI mapping not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "S-NSSAI mapping deleted successfully" }),
    ))
}
