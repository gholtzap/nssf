use chrono::Utc;
use mongodb::Database;
use bson::doc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::types::db::NssaiAvailabilitySubscription;
use crate::types::nssaiavailability::{
    NssfEventSubscriptionCreateData, NssfEventSubscriptionCreatedData, PatchItem, PatchOp,
};

pub async fn create_subscription(
    data: NssfEventSubscriptionCreateData,
    db: &Database,
) -> AppResult<NssfEventSubscriptionCreatedData> {
    let collection = db.collection::<NssaiAvailabilitySubscription>(
        "nssai_availability_subscriptions",
    );

    let subscription_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let subscription = NssaiAvailabilitySubscription {
        subscription_id: subscription_id.clone(),
        nf_instance_id: data.amf_id.clone().unwrap_or_default(),
        subscription_data: crate::types::db::NssaiAvailabilitySubscriptionData {
            tai: data.tai_list.as_ref().and_then(|l| l.first()).cloned().unwrap_or(
                crate::types::common::Tai {
                    plmn_id: crate::types::common::PlmnId {
                        mcc: "000".to_string(),
                        mnc: "00".to_string(),
                    },
                    tac: "0000".to_string(),
                },
            ),
            supported_snssai_list: None,
        },
        notification_uri: data.nf_nssai_availability_uri.clone(),
        supported_features: data.supported_features.clone(),
        expiry_time: data.expiry.clone(),
        created_at: now,
        updated_at: now,
    };

    collection.insert_one(&subscription).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create subscription: {}", e))
    })?;

    Ok(NssfEventSubscriptionCreatedData {
        subscription_id,
        expiry: data.expiry,
        authorized_nssai_availability_data: None,
        supported_features: data.supported_features,
    })
}

pub async fn get_subscription(
    subscription_id: &str,
    db: &Database,
) -> AppResult<NssaiAvailabilitySubscription> {
    let collection = db.collection::<NssaiAvailabilitySubscription>(
        "nssai_availability_subscriptions",
    );

    let subscription = collection
        .find_one(doc! { "subscriptionId": subscription_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query subscription: {}", e))
        })?;

    match subscription {
        Some(s) => Ok(s),
        None => Err(AppError::NotFound("Subscription not found".to_string())),
    }
}

pub async fn update_subscription(
    subscription_id: &str,
    patches: Vec<PatchItem>,
    db: &Database,
) -> AppResult<NssfEventSubscriptionCreatedData> {
    let collection = db.collection::<NssaiAvailabilitySubscription>(
        "nssai_availability_subscriptions",
    );

    let existing = collection
        .find_one(doc! { "subscriptionId": subscription_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query subscription: {}", e))
        })?;

    let existing = match existing {
        Some(s) => s,
        None => return Err(AppError::NotFound("Subscription not found".to_string())),
    };

    let mut update_doc = doc! {};

    for patch in &patches {
        let field = patch.path.trim_start_matches('/').replace('/', ".");

        match patch.op {
            PatchOp::Replace | PatchOp::Add => {
                if let Some(value) = &patch.value {
                    let bson_value = bson::to_bson(value).map_err(|e| {
                        AppError::BadRequest(format!("Invalid patch value: {}", e))
                    })?;
                    update_doc.insert(field, bson_value);
                }
            }
            PatchOp::Remove => {
                update_doc.insert(field, bson::Bson::Null);
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unsupported patch operation: {:?}",
                    patch.op
                )));
            }
        }
    }

    update_doc.insert("updatedAt", bson::to_bson(&Utc::now()).unwrap());

    collection
        .update_one(
            doc! { "subscriptionId": subscription_id },
            doc! { "$set": update_doc },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update subscription: {}", e))
        })?;

    Ok(NssfEventSubscriptionCreatedData {
        subscription_id: subscription_id.to_string(),
        expiry: existing.expiry_time,
        authorized_nssai_availability_data: None,
        supported_features: existing.supported_features,
    })
}

pub async fn delete_subscription(
    subscription_id: &str,
    db: &Database,
) -> AppResult<()> {
    let collection = db.collection::<NssaiAvailabilitySubscription>(
        "nssai_availability_subscriptions",
    );

    let result = collection
        .delete_one(doc! { "subscriptionId": subscription_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to delete subscription: {}", e))
        })?;

    if result.deleted_count == 0 {
        return Err(AppError::NotFound("Subscription not found".to_string()));
    }

    Ok(())
}
