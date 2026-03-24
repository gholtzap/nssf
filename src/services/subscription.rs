use bson::doc;
use mongodb::Database;
use tracing::debug;

use crate::errors::{AppError, AppResult};
use crate::types::common::{PlmnId, Snssai};
use crate::types::db::{SubscribedSnssai, UeSubscription};

pub async fn get_subscription(
    db: &Database,
    supi: &str,
    plmn_id: &PlmnId,
) -> AppResult<Option<UeSubscription>> {
    let collection = db.collection::<UeSubscription>("subscriptions");

    let subscription = collection
        .find_one(doc! {
            "supi": supi,
            "plmnId.mcc": &plmn_id.mcc,
            "plmnId.mnc": &plmn_id.mnc,
        })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query subscription: {}", e))
        })?;

    debug!(
        supi = supi,
        mcc = %plmn_id.mcc,
        mnc = %plmn_id.mnc,
        found = subscription.is_some(),
        "Fetched UE subscription"
    );

    Ok(subscription)
}

pub fn get_subscribed_snssais(subscription: &UeSubscription) -> Vec<Snssai> {
    subscription
        .subscribed_snssais
        .iter()
        .map(|s| s.subscribed_snssai.clone())
        .collect()
}

pub fn get_default_snssai(subscription: &UeSubscription) -> Option<Snssai> {
    if let Some(ref default) = subscription.default_snssai {
        return Some(default.clone());
    }

    subscription
        .subscribed_snssais
        .iter()
        .find(|s| s.default_indication == Some(true))
        .map(|s| s.subscribed_snssai.clone())
}

pub fn get_subscribed_nssrg_list(subscribed: &SubscribedSnssai) -> Vec<String> {
    subscribed
        .subscribed_nssrg_list
        .clone()
        .unwrap_or_default()
}
