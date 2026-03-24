use bson::doc;
use chrono::{Datelike, Utc};
use futures::TryStreamExt;
use mongodb::Database;
use tracing::debug;

use crate::errors::{AppError, AppResult};
use crate::types::common::{PlmnId, Snssai, Tai};
use crate::types::db::{NsiConfiguration, SliceConfiguration, SlicePolicy, TimeWindow, UeSubscription};
use crate::validation::snssai_eq;

pub fn check_subscription(snssai: &Snssai, subscription: &UeSubscription) -> bool {
    subscription
        .subscribed_snssais
        .iter()
        .any(|s| snssai_eq(snssai, &s.subscribed_snssai))
}

pub async fn check_slice_availability(
    snssai: &Snssai,
    plmn: &PlmnId,
    db: &Database,
) -> AppResult<bool> {
    let collection = db.collection::<SliceConfiguration>("slices");

    let cursor = collection
        .find(doc! {
            "snssai.sst": snssai.sst as i32,
            "plmnId.mcc": &plmn.mcc,
            "plmnId.mnc": &plmn.mnc,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query slices: {}", e)))?;

    let slices: Vec<SliceConfiguration> = cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read slices: {}", e)))?;

    let available = slices.iter().any(|s| snssai_eq(snssai, &s.snssai));

    debug!(
        sst = snssai.sst,
        sd = ?snssai.sd,
        available,
        "Checked slice availability"
    );

    Ok(available)
}

pub fn tai_eq(a: &Tai, b: &Tai) -> bool {
    a.plmn_id.mcc == b.plmn_id.mcc
        && a.plmn_id.mnc == b.plmn_id.mnc
        && a.tac.to_ascii_lowercase() == b.tac.to_ascii_lowercase()
}

pub async fn check_tai_availability(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<bool> {
    let collection = db.collection::<SliceConfiguration>("slices");

    let cursor = collection
        .find(doc! {
            "snssai.sst": snssai.sst as i32,
            "plmnId.mcc": &tai.plmn_id.mcc,
            "plmnId.mnc": &tai.plmn_id.mnc,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query slices: {}", e)))?;

    let slices: Vec<SliceConfiguration> = cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read slices: {}", e)))?;

    for slice in &slices {
        if !snssai_eq(snssai, &slice.snssai) {
            continue;
        }

        match &slice.tai_list {
            None => return Ok(true),
            Some(tai_list) if tai_list.is_empty() => return Ok(true),
            Some(tai_list) => {
                if tai_list.iter().any(|t| tai_eq(t, tai)) {
                    return Ok(true);
                }
            }
        }
    }

    debug!(
        sst = snssai.sst,
        sd = ?snssai.sd,
        tac = %tai.tac,
        "S-NSSAI not available in TAI"
    );
    Ok(false)
}

fn is_within_time_window(window: &TimeWindow, now: &chrono::DateTime<Utc>) -> bool {
    let start = match chrono::DateTime::parse_from_rfc3339(&window.start_time) {
        Ok(t) => t.with_timezone(&Utc),
        Err(_) => return false,
    };
    let end = match chrono::DateTime::parse_from_rfc3339(&window.end_time) {
        Ok(t) => t.with_timezone(&Utc),
        Err(_) => return false,
    };

    if let Some(ref days) = window.days_of_week {
        let day = now.weekday().num_days_from_monday() as u8;
        if !days.contains(&day) {
            return false;
        }
    }

    *now >= start && *now < end
}

pub async fn evaluate_policy(
    snssai: &Snssai,
    plmn: &PlmnId,
    tai: &Tai,
    db: &Database,
) -> AppResult<bool> {
    let collection = db.collection::<SlicePolicy>("policies");

    let cursor = collection
        .find(doc! {
            "snssai.sst": snssai.sst as i32,
            "plmnId.mcc": &plmn.mcc,
            "plmnId.mnc": &plmn.mnc,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query policies: {}", e)))?;

    let policies: Vec<SlicePolicy> = cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read policies: {}", e)))?;

    let matching: Vec<&SlicePolicy> = policies
        .iter()
        .filter(|p| snssai_eq(snssai, &p.snssai))
        .collect();

    if matching.is_empty() {
        debug!(
            sst = snssai.sst,
            sd = ?snssai.sd,
            "No policy found, allowing S-NSSAI by default"
        );
        return Ok(true);
    }

    let now = Utc::now();

    for policy in &matching {
        if !policy.enabled {
            debug!(policy_id = %policy.policy_id, "Policy is disabled, denying S-NSSAI");
            return Ok(false);
        }

        if let Some(ref windows) = policy.allowed_time_windows {
            if !windows.is_empty() && !windows.iter().any(|w| is_within_time_window(w, &now)) {
                debug!(
                    policy_id = %policy.policy_id,
                    "Current time not within allowed time windows"
                );
                return Ok(false);
            }
        }

        if let Some(ref windows) = policy.denied_time_windows {
            if windows.iter().any(|w| is_within_time_window(w, &now)) {
                debug!(
                    policy_id = %policy.policy_id,
                    "Current time within denied time window"
                );
                return Ok(false);
            }
        }

        if let Some(ref allowed_tais) = policy.allowed_tai_list {
            if !allowed_tais.is_empty() && !allowed_tais.iter().any(|t| tai_eq(t, tai)) {
                debug!(
                    policy_id = %policy.policy_id,
                    "TAI not in allowed TAI list"
                );
                return Ok(false);
            }
        }

        if let Some(ref denied_tais) = policy.denied_tai_list {
            if denied_tais.iter().any(|t| tai_eq(t, tai)) {
                debug!(
                    policy_id = %policy.policy_id,
                    "TAI in denied TAI list"
                );
                return Ok(false);
            }
        }

        if let Some(max_load) = policy.max_load_level {
            if check_load_exceeded(snssai, plmn, max_load, db).await? {
                debug!(
                    policy_id = %policy.policy_id,
                    max_load_level = max_load,
                    "NSI load exceeds policy max"
                );
                return Ok(false);
            }
        }
    }

    Ok(true)
}

async fn check_load_exceeded(
    snssai: &Snssai,
    plmn: &PlmnId,
    max_load: u32,
    db: &Database,
) -> AppResult<bool> {
    let collection = db.collection::<NsiConfiguration>("nsi");

    let cursor = collection
        .find(doc! {
            "snssai.sst": snssai.sst as i32,
            "plmnId.mcc": &plmn.mcc,
            "plmnId.mnc": &plmn.mnc,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query NSI: {}", e)))?;

    let nsis: Vec<NsiConfiguration> = cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read NSI: {}", e)))?;

    for nsi in &nsis {
        if snssai_eq(snssai, &nsi.snssai) {
            if let Some(load) = nsi.load_level {
                if load >= max_load {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

pub async fn is_snssai_allowed(
    snssai: &Snssai,
    plmn: &PlmnId,
    tai: &Tai,
    subscription: &UeSubscription,
    db: &Database,
) -> AppResult<bool> {
    if !check_subscription(snssai, subscription) {
        debug!(
            sst = snssai.sst,
            sd = ?snssai.sd,
            "S-NSSAI not in UE subscription"
        );
        return Ok(false);
    }

    if !check_slice_availability(snssai, plmn, db).await? {
        return Ok(false);
    }

    if !check_tai_availability(snssai, tai, db).await? {
        return Ok(false);
    }

    if !evaluate_policy(snssai, plmn, tai, db).await? {
        return Ok(false);
    }

    debug!(
        sst = snssai.sst,
        sd = ?snssai.sd,
        "S-NSSAI is allowed"
    );
    Ok(true)
}
