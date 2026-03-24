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
use crate::types::db::NsiConfiguration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsiQuery {
    pub sst: Option<u8>,
    pub sd: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
}

pub async fn list_nsi(
    State(state): State<AppState>,
    Query(query): Query<NsiQuery>,
) -> AppResult<Json<Vec<NsiConfiguration>>> {
    let collection = state.db.collection::<NsiConfiguration>("nsi");

    let filter = match (query.sst, &query.mcc, &query.mnc) {
        (Some(sst), Some(mcc), Some(mnc)) => {
            let mut f = doc! {
                "snssai.sst": sst as i32,
                "plmnId.mcc": mcc,
                "plmnId.mnc": mnc,
            };
            if let Some(sd) = &query.sd {
                f.insert("snssai.sd", sd);
            }
            f
        }
        _ => doc! {},
    };

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query NSI configurations: {}", e))
    })?;

    let nsis: Vec<NsiConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect NSI configurations: {}", e))
    })?;

    Ok(Json(nsis))
}

pub async fn get_nsi(
    State(state): State<AppState>,
    Path(nsi_id): Path<String>,
) -> AppResult<Json<NsiConfiguration>> {
    let collection = state.db.collection::<NsiConfiguration>("nsi");

    let nsi = collection
        .find_one(doc! { "nsiId": &nsi_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query NSI configuration: {}", e))
        })?;

    match nsi {
        Some(n) => Ok(Json(n)),
        None => Err(AppError::NotFound(
            "NSI configuration not found".to_string(),
        )),
    }
}

pub async fn create_nsi(
    State(state): State<AppState>,
    Json(nsi): Json<NsiConfiguration>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<NsiConfiguration>("nsi");

    let existing = collection
        .find_one(doc! { "nsiId": &nsi.nsi_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing NSI: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "NSI configuration already exists for this NSI ID".to_string(),
        ));
    }

    collection.insert_one(nsi).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create NSI configuration: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "NSI configuration created successfully" })),
    ))
}

pub async fn update_nsi(
    State(state): State<AppState>,
    Path(nsi_id): Path<String>,
    Json(updates): Json<NsiConfiguration>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NsiConfiguration>("nsi");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(doc! { "nsiId": &nsi_id }, doc! { "$set": update_doc })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update NSI configuration: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(
            "NSI configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSI configuration updated successfully" }),
    ))
}

pub async fn delete_nsi(
    State(state): State<AppState>,
    Path(nsi_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<NsiConfiguration>("nsi");

    let result = collection
        .delete_one(doc! { "nsiId": &nsi_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete NSI configuration: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(
            "NSI configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "NSI configuration deleted successfully" }),
    ))
}
