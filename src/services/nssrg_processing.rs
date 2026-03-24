use bson::doc;
use futures::TryStreamExt;
use mongodb::Database;
use tracing::debug;

use crate::errors::{AppError, AppResult};
use crate::types::common::{Snssai, Tai};
use crate::types::db::NssrgConfiguration;
use crate::validation::snssai_eq;

use super::allowed_nssai::tai_eq;

fn nssrg_matches_snssai(nssrg: &NssrgConfiguration, snssai: &Snssai) -> bool {
    nssrg.snssai_list.iter().any(|s| snssai_eq(s, snssai))
}

fn nssrg_matches_tai(nssrg: &NssrgConfiguration, tai: &Tai) -> bool {
    match &nssrg.tai_list {
        None => true,
        Some(tai_list) if tai_list.is_empty() => true,
        Some(tai_list) => tai_list.iter().any(|t| tai_eq(t, tai)),
    }
}

fn nssrg_has_capacity(nssrg: &NssrgConfiguration) -> bool {
    match (nssrg.max_ue_count, nssrg.current_ue_count) {
        (Some(max), Some(current)) => current < max,
        (Some(_max), None) => true,
        (None, _) => true,
    }
}

fn nssrg_load_fraction(nssrg: &NssrgConfiguration) -> f64 {
    match (nssrg.current_ue_count, nssrg.max_ue_count) {
        (Some(current), Some(max)) if max > 0 => current as f64 / max as f64,
        (None, Some(_)) => 0.0,
        _ => 0.0,
    }
}

async fn find_matching_nssrgs(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<Vec<NssrgConfiguration>> {
    let collection = db.collection::<NssrgConfiguration>("nssrg_configurations");

    let cursor = collection
        .find(doc! { "enabled": true })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to query NSSRG configurations: {}",
                e
            ))
        })?;

    let nssrgs: Vec<NssrgConfiguration> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read NSSRG configurations: {}", e))
    })?;

    let matching: Vec<NssrgConfiguration> = nssrgs
        .into_iter()
        .filter(|n| nssrg_matches_snssai(n, snssai) && nssrg_matches_tai(n, tai))
        .collect();

    Ok(matching)
}

pub async fn assign_nssrg(
    snssai: &Snssai,
    tai: &Tai,
    db: &Database,
) -> AppResult<Option<String>> {
    let mut matching = find_matching_nssrgs(snssai, tai, db).await?;

    if matching.is_empty() {
        debug!(
            sst = snssai.sst,
            sd = ?snssai.sd,
            "No NSSRG configuration found for S-NSSAI"
        );
        return Ok(None);
    }

    matching.retain(|n| nssrg_has_capacity(n));

    if matching.is_empty() {
        debug!(
            sst = snssai.sst,
            sd = ?snssai.sd,
            "All matching NSSRGs at capacity"
        );
        return Ok(None);
    }

    matching.sort_by(|a, b| {
        let load_cmp = nssrg_load_fraction(a)
            .partial_cmp(&nssrg_load_fraction(b))
            .unwrap_or(std::cmp::Ordering::Equal);

        let priority_cmp = a
            .priority
            .unwrap_or(u32::MAX)
            .cmp(&b.priority.unwrap_or(u32::MAX));

        priority_cmp.then(load_cmp)
    });

    let selected = &matching[0];
    increment_nssrg_count(&selected.nssrg_id, db).await?;

    debug!(
        nssrg_id = %selected.nssrg_id,
        sst = snssai.sst,
        sd = ?snssai.sd,
        "Assigned UE to NSSRG"
    );

    Ok(Some(selected.nssrg_id.clone()))
}

async fn increment_nssrg_count(nssrg_id: &str, db: &Database) -> AppResult<()> {
    let collection = db.collection::<NssrgConfiguration>("nssrg_configurations");

    collection
        .update_one(
            doc! { "nssrgId": nssrg_id },
            doc! { "$inc": { "currentUeCount": 1_i64 } },
        )
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to increment NSSRG UE count for nssrg_id {}: {}",
                nssrg_id, e
            ))
        })?;

    Ok(())
}
