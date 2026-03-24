use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use serde::Deserialize;

use crate::errors::{AppError, AppResult};
use crate::types::AppState;
use crate::types::db::SliceConfiguration;

#[derive(Debug, Deserialize)]
pub struct SliceQuery {
    pub sst: Option<u8>,
    pub sd: Option<String>,
}

pub async fn list_slices(
    State(state): State<AppState>,
    Query(query): Query<SliceQuery>,
) -> AppResult<Json<Vec<SliceConfiguration>>> {
    let collection = state.db.collection::<SliceConfiguration>("slices");

    let filter = match (query.sst, &query.sd) {
        (Some(sst), Some(sd)) => doc! { "snssai.sst": sst as i32, "snssai.sd": sd },
        (Some(sst), None) => doc! { "snssai.sst": sst as i32 },
        _ => doc! {},
    };

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query slices: {}", e))
    })?;

    let slices: Vec<SliceConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect slices: {}", e))
    })?;

    Ok(Json(slices))
}

pub async fn create_slice(
    State(state): State<AppState>,
    Json(slice): Json<SliceConfiguration>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<SliceConfiguration>("slices");

    collection.insert_one(slice).await.map_err(|e| {
        if e.to_string().contains("E11000") {
            AppError::BadRequest(
                "Slice configuration already exists for this S-NSSAI and PLMN".to_string(),
            )
        } else {
            AppError::InternalServerError(format!("Failed to create slice: {}", e))
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Slice configuration created successfully" })),
    ))
}

pub async fn update_slice(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut updates): Json<SliceConfiguration>,
) -> AppResult<Json<serde_json::Value>> {
    let oid = ObjectId::parse_str(&id)
        .map_err(|_| AppError::BadRequest(format!("Invalid ObjectId: {}", id)))?;

    let collection = state.db.collection::<SliceConfiguration>("slices");

    updates.id = None;

    let result = collection
        .replace_one(doc! { "_id": oid }, updates)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update slice: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(
            "Slice configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "Slice configuration updated successfully" }),
    ))
}

pub async fn delete_slice(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let oid = ObjectId::parse_str(&id)
        .map_err(|_| AppError::BadRequest(format!("Invalid ObjectId: {}", id)))?;

    let collection = state.db.collection::<SliceConfiguration>("slices");

    let result = collection
        .delete_one(doc! { "_id": oid })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete slice: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(
            "Slice configuration not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "Slice configuration deleted successfully" }),
    ))
}
