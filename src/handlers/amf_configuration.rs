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
use crate::types::db::{AmfInstanceConfig, AmfServiceSetConfig, AmfSetConfig};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfSetQuery {
    pub mcc: Option<String>,
    pub mnc: Option<String>,
}

pub async fn list_amf_sets(
    State(state): State<AppState>,
    Query(query): Query<AmfSetQuery>,
) -> AppResult<Json<Vec<AmfSetConfig>>> {
    let collection = state.db.collection::<AmfSetConfig>("amf_sets");

    let filter = match (&query.mcc, &query.mnc) {
        (Some(mcc), Some(mnc)) => doc! { "plmnId.mcc": mcc, "plmnId.mnc": mnc },
        _ => doc! {},
    };

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query AMF sets: {}", e))
    })?;

    let sets: Vec<AmfSetConfig> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect AMF sets: {}", e))
    })?;

    Ok(Json(sets))
}

pub async fn get_amf_set(
    State(state): State<AppState>,
    Path(amf_set_id): Path<String>,
) -> AppResult<Json<AmfSetConfig>> {
    let collection = state.db.collection::<AmfSetConfig>("amf_sets");

    let set = collection
        .find_one(doc! { "amfSetId": &amf_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query AMF set: {}", e))
        })?;

    match set {
        Some(s) => Ok(Json(s)),
        None => Err(AppError::NotFound("AMF set not found".to_string())),
    }
}

pub async fn create_amf_set(
    State(state): State<AppState>,
    Json(set): Json<AmfSetConfig>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<AmfSetConfig>("amf_sets");

    let existing = collection
        .find_one(doc! { "amfSetId": &set.amf_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing AMF set: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "AMF set already exists for this AMF set ID".to_string(),
        ));
    }

    collection.insert_one(set).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create AMF set: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "AMF set created successfully" })),
    ))
}

pub async fn update_amf_set(
    State(state): State<AppState>,
    Path(amf_set_id): Path<String>,
    Json(updates): Json<AmfSetConfig>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfSetConfig>("amf_sets");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(
            doc! { "amfSetId": &amf_set_id },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update AMF set: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound("AMF set not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF set updated successfully" }),
    ))
}

pub async fn delete_amf_set(
    State(state): State<AppState>,
    Path(amf_set_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfSetConfig>("amf_sets");

    let result = collection
        .delete_one(doc! { "amfSetId": &amf_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete AMF set: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound("AMF set not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF set deleted successfully" }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfServiceSetQuery {
    pub amf_set_id: Option<String>,
}

pub async fn list_amf_service_sets(
    State(state): State<AppState>,
    Query(query): Query<AmfServiceSetQuery>,
) -> AppResult<Json<Vec<AmfServiceSetConfig>>> {
    let collection = state.db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let filter = match &query.amf_set_id {
        Some(amf_set_id) => doc! { "amfSetId": amf_set_id },
        None => doc! {},
    };

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query AMF service sets: {}", e))
    })?;

    let sets: Vec<AmfServiceSetConfig> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect AMF service sets: {}", e))
    })?;

    Ok(Json(sets))
}

pub async fn get_amf_service_set(
    State(state): State<AppState>,
    Path(amf_service_set_id): Path<String>,
) -> AppResult<Json<AmfServiceSetConfig>> {
    let collection = state.db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let set = collection
        .find_one(doc! { "amfServiceSetId": &amf_service_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query AMF service set: {}", e))
        })?;

    match set {
        Some(s) => Ok(Json(s)),
        None => Err(AppError::NotFound(
            "AMF service set not found".to_string(),
        )),
    }
}

pub async fn create_amf_service_set(
    State(state): State<AppState>,
    Json(set): Json<AmfServiceSetConfig>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let existing = collection
        .find_one(doc! { "amfServiceSetId": &set.amf_service_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to check existing AMF service set: {}",
                e
            ))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "AMF service set already exists for this ID".to_string(),
        ));
    }

    collection.insert_one(set).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create AMF service set: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "AMF service set created successfully" })),
    ))
}

pub async fn update_amf_service_set(
    State(state): State<AppState>,
    Path(amf_service_set_id): Path<String>,
    Json(updates): Json<AmfServiceSetConfig>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(
            doc! { "amfServiceSetId": &amf_service_set_id },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update AMF service set: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(
            "AMF service set not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF service set updated successfully" }),
    ))
}

pub async fn delete_amf_service_set(
    State(state): State<AppState>,
    Path(amf_service_set_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let result = collection
        .delete_one(doc! { "amfServiceSetId": &amf_service_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete AMF service set: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(
            "AMF service set not found".to_string(),
        ));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF service set deleted successfully" }),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfInstanceQuery {
    pub amf_set_id: Option<String>,
}

pub async fn list_amf_instances(
    State(state): State<AppState>,
    Query(query): Query<AmfInstanceQuery>,
) -> AppResult<Json<Vec<AmfInstanceConfig>>> {
    let collection = state.db.collection::<AmfInstanceConfig>("amf_instances");

    let filter = match &query.amf_set_id {
        Some(amf_set_id) => doc! { "amfSetId": amf_set_id },
        None => doc! {},
    };

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query AMF instances: {}", e))
    })?;

    let instances: Vec<AmfInstanceConfig> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect AMF instances: {}", e))
    })?;

    Ok(Json(instances))
}

pub async fn get_amf_instance(
    State(state): State<AppState>,
    Path(nf_instance_id): Path<String>,
) -> AppResult<Json<AmfInstanceConfig>> {
    let collection = state.db.collection::<AmfInstanceConfig>("amf_instances");

    let instance = collection
        .find_one(doc! { "nfInstanceId": &nf_instance_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query AMF instance: {}", e))
        })?;

    match instance {
        Some(i) => Ok(Json(i)),
        None => Err(AppError::NotFound("AMF instance not found".to_string())),
    }
}

pub async fn create_amf_instance(
    State(state): State<AppState>,
    Json(instance): Json<AmfInstanceConfig>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<AmfInstanceConfig>("amf_instances");

    let existing = collection
        .find_one(doc! { "nfInstanceId": &instance.nf_instance_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to check existing AMF instance: {}",
                e
            ))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "AMF instance already exists for this NF instance ID".to_string(),
        ));
    }

    collection.insert_one(instance).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create AMF instance: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "AMF instance created successfully" })),
    ))
}

pub async fn update_amf_instance(
    State(state): State<AppState>,
    Path(nf_instance_id): Path<String>,
    Json(updates): Json<AmfInstanceConfig>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfInstanceConfig>("amf_instances");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(
            doc! { "nfInstanceId": &nf_instance_id },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update AMF instance: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound("AMF instance not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF instance updated successfully" }),
    ))
}

pub async fn delete_amf_instance(
    State(state): State<AppState>,
    Path(nf_instance_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<AmfInstanceConfig>("amf_instances");

    let result = collection
        .delete_one(doc! { "nfInstanceId": &nf_instance_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete AMF instance: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound("AMF instance not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({ "message": "AMF instance deleted successfully" }),
    ))
}
