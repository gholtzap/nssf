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
use crate::types::db::UeSubscription;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionQuery {
    pub supi: Option<String>,
    pub mcc: Option<String>,
    pub mnc: Option<String>,
}

pub async fn list_subscriptions(
    State(state): State<AppState>,
    Query(query): Query<SubscriptionQuery>,
) -> AppResult<Json<Vec<UeSubscription>>> {
    let collection = state.db.collection::<UeSubscription>("subscriptions");

    let mut filter = doc! {};
    if let Some(supi) = &query.supi {
        filter.insert("supi", supi);
    }
    if let Some(mcc) = &query.mcc {
        filter.insert("plmnId.mcc", mcc);
    }
    if let Some(mnc) = &query.mnc {
        filter.insert("plmnId.mnc", mnc);
    }

    let cursor = collection.find(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to query subscriptions: {}", e))
    })?;

    let subscriptions: Vec<UeSubscription> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect subscriptions: {}", e))
    })?;

    Ok(Json(subscriptions))
}

pub async fn get_subscription(
    State(state): State<AppState>,
    Path(supi): Path<String>,
) -> AppResult<Json<Vec<UeSubscription>>> {
    let collection = state.db.collection::<UeSubscription>("subscriptions");

    let cursor = collection
        .find(doc! { "supi": &supi })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query subscription: {}", e))
        })?;

    let subscriptions: Vec<UeSubscription> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to collect subscriptions: {}", e))
    })?;

    if subscriptions.is_empty() {
        return Err(AppError::NotFound(format!(
            "No subscriptions found for SUPI: {}",
            supi
        )));
    }

    Ok(Json(subscriptions))
}

pub async fn create_subscription(
    State(state): State<AppState>,
    Json(subscription): Json<UeSubscription>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let collection = state.db.collection::<UeSubscription>("subscriptions");

    let existing = collection
        .find_one(doc! {
            "supi": &subscription.supi,
            "plmnId.mcc": &subscription.plmn_id.mcc,
            "plmnId.mnc": &subscription.plmn_id.mnc,
        })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to check existing subscription: {}", e))
        })?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "Subscription already exists for this SUPI and PLMN".to_string(),
        ));
    }

    collection.insert_one(subscription).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create subscription: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Subscription created successfully" })),
    ))
}

pub async fn update_subscription(
    State(state): State<AppState>,
    Path(supi): Path<String>,
    Json(updates): Json<UeSubscription>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<UeSubscription>("subscriptions");

    let update_doc = bson::to_document(&updates).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize updates: {}", e))
    })?;

    let result = collection
        .update_one(
            doc! {
                "supi": &supi,
                "plmnId.mcc": &updates.plmn_id.mcc,
                "plmnId.mnc": &updates.plmn_id.mnc,
            },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update subscription: {}", e))
        })?;

    if result.matched_count == 0 {
        return Err(AppError::NotFound(format!(
            "Subscription not found for SUPI: {}",
            supi
        )));
    }

    Ok(Json(
        serde_json::json!({ "message": "Subscription updated successfully" }),
    ))
}

pub async fn delete_subscription(
    State(state): State<AppState>,
    Path(supi): Path<String>,
    Query(query): Query<SubscriptionQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let collection = state.db.collection::<UeSubscription>("subscriptions");

    let filter = match (&query.mcc, &query.mnc) {
        (Some(mcc), Some(mnc)) => doc! {
            "supi": &supi,
            "plmnId.mcc": mcc,
            "plmnId.mnc": mnc,
        },
        _ => doc! { "supi": &supi },
    };

    let result = collection.delete_many(filter).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to delete subscription: {}", e))
    })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound(format!(
            "Subscription not found for SUPI: {}",
            supi
        )));
    }

    Ok(Json(
        serde_json::json!({ "message": "Subscription deleted successfully" }),
    ))
}
