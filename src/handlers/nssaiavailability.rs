use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};

use crate::errors::AppResult;
use crate::services::nssai_availability;
use crate::types::AppState;
use crate::types::nssaiavailability::{
    NssfEventSubscriptionCreateData, NssfEventSubscriptionCreatedData, PatchItem,
};

pub async fn create_subscription(
    State(state): State<AppState>,
    Json(data): Json<NssfEventSubscriptionCreateData>,
) -> AppResult<(StatusCode, HeaderMap, Json<NssfEventSubscriptionCreatedData>)> {
    let result = nssai_availability::create_subscription(data, &state.db).await?;
    let mut headers = HeaderMap::new();
    let location = format!(
        "/nnssf-nssaiavailability/v1/subscriptions/{}",
        result.subscription_id
    );
    if let Ok(val) = HeaderValue::from_str(&location) {
        headers.insert("Location", val);
    }
    Ok((StatusCode::CREATED, headers, Json(result)))
}

pub async fn get_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<String>,
) -> AppResult<Json<NssfEventSubscriptionCreatedData>> {
    let subscription =
        nssai_availability::get_subscription(&subscription_id, &state.db).await?;

    Ok(Json(NssfEventSubscriptionCreatedData {
        subscription_id: subscription.subscription_id,
        expiry: subscription.expiry_time,
        authorized_nssai_availability_data: None,
        supported_features: subscription.supported_features,
    }))
}

pub async fn update_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<String>,
    Json(patches): Json<Vec<PatchItem>>,
) -> AppResult<Json<NssfEventSubscriptionCreatedData>> {
    let result =
        nssai_availability::update_subscription(&subscription_id, patches, &state.db).await?;
    Ok(Json(result))
}

pub async fn delete_subscription(
    State(state): State<AppState>,
    Path(subscription_id): Path<String>,
) -> AppResult<StatusCode> {
    nssai_availability::delete_subscription(&subscription_id, &state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
