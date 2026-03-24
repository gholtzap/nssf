use bson::doc;
use futures::TryStreamExt;
use mongodb::Database;
use tracing::debug;

use crate::errors::{AppError, AppResult};
use crate::types::common::{Snssai, Tai};
use crate::types::db::NsagConfiguration;
use crate::validation::snssai_eq;

use super::allowed_nssai::tai_eq;

fn nsag_matches_snssai(nsag: &NsagConfiguration, snssai: &Snssai) -> bool {
    nsag.snssai_list.iter().any(|s| snssai_eq(s, snssai))
}

fn nsag_matches_tai(nsag: &NsagConfiguration, tai: &Tai) -> bool {
    match &nsag.tai_list {
        None => true,
        Some(tai_list) if tai_list.is_empty() => true,
        Some(tai_list) => tai_list.iter().any(|t| tai_eq(t, tai)),
    }
}

fn nsag_has_capacity(nsag: &NsagConfiguration) -> bool {
    match (nsag.max_ue_count, nsag.current_ue_count) {
        (Some(max), Some(current)) => current < max,
        (Some(_max), None) => true,
        (None, _) => true,
    }
}

async fn find_matching_nsags(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<Vec<NsagConfiguration>> {
    let collection = db.collection::<NsagConfiguration>("nsag_configurations");

    let cursor = collection
        .find(doc! { "enabled": true })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to query NSAG configurations: {}",
                e
            ))
        })?;

    let nsags: Vec<NsagConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read NSAG configurations: {}", e))
    })?;

    let matching: Vec<NsagConfiguration> = nsags
        .into_iter()
        .filter(|n| nsag_matches_snssai(n, snssai) && nsag_matches_tai(n, tai))
        .collect();

    Ok(matching)
}

pub async fn check_nsag_admission(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<bool> {
    let mut matching = find_matching_nsags(snssai, tai, db).await?;

    if matching.is_empty() {
        debug!(
            sst = snssai.sst,
            sd = ?snssai.sd,
            "No NSAG configuration found, admission allowed by default"
        );
        return Ok(true);
    }

    matching.sort_by_key(|n| n.priority.unwrap_or(u32::MAX));

    let admitted = matching.iter().any(|n| nsag_has_capacity(n));

    debug!(
        sst = snssai.sst,
        sd = ?snssai.sd,
        matching_count = matching.len(),
        admitted,
        "NSAG admission check"
    );

    Ok(admitted)
}

pub async fn admit_to_nsag(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<Option<u32>> {
    let mut matching = find_matching_nsags(snssai, tai, db).await?;

    if matching.is_empty() {
        return Ok(None);
    }

    matching.sort_by_key(|n| n.priority.unwrap_or(u32::MAX));

    for nsag in &matching {
        if nsag_has_capacity(nsag) {
            increment_nsag_count(nsag.nsag_id, db).await?;
            debug!(nsag_id = nsag.nsag_id, "UE admitted to NSAG");
            return Ok(Some(nsag.nsag_id));
        }
    }

    debug!(
        sst = snssai.sst,
        sd = ?snssai.sd,
        "All matching NSAGs at capacity"
    );
    Ok(None)
}

pub async fn increment_nsag_count(nsag_id: u32, db: &Database) -> AppResult<()> {
    let collection = db.collection::<NsagConfiguration>("nsag_configurations");

    collection
        .update_one(
            doc! { "nsagId": nsag_id },
            doc! { "$inc": { "currentUeCount": 1_i64 } },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to increment NSAG UE count for nsag_id {}: {}",
                nsag_id, e
            ))
        })?;

    Ok(())
}
