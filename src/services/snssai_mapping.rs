use bson::doc;
use futures::TryStreamExt;
use mongodb::Database;
use tracing::debug;

use crate::errors::{AppError, AppResult};
use crate::types::common::{PlmnId, Snssai, Tai};
use crate::types::db::SnssaiMapping;
use crate::validation::snssai_eq;

use super::allowed_nssai::tai_eq;

pub async fn map_snssai(
    serving_snssai: &Snssai,
    serving_plmn: &PlmnId,
    home_plmn: &PlmnId,
    db: &Database,
) -> AppResult<Option<Snssai>> {
    let collection = db.collection::<SnssaiMapping>("snssai_mappings");

    let cursor = collection
        .find(doc! {
            "servingSnssai.sst": serving_snssai.sst as i32,
            "servingPlmnId.mcc": &serving_plmn.mcc,
            "servingPlmnId.mnc": &serving_plmn.mnc,
            "homePlmnId.mcc": &home_plmn.mcc,
            "homePlmnId.mnc": &home_plmn.mnc,
        })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query snssai_mappings: {}", e))
        })?;

    let mappings: Vec<SnssaiMapping> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read snssai_mappings: {}", e))
    })?;

    let result = mappings
        .iter()
        .find(|m| snssai_eq(serving_snssai, &m.serving_snssai))
        .map(|m| m.home_snssai.clone());

    debug!(
        sst = serving_snssai.sst,
        sd = ?serving_snssai.sd,
        serving_mcc = %serving_plmn.mcc,
        serving_mnc = %serving_plmn.mnc,
        home_mcc = %home_plmn.mcc,
        home_mnc = %home_plmn.mnc,
        found = result.is_some(),
        "S-NSSAI mapping lookup"
    );

    Ok(result)
}

pub async fn map_snssai_with_tai(
    serving_snssai: &Snssai,
    serving_plmn: &PlmnId,
    home_plmn: &PlmnId,
    current_tai: &Tai,
    db: &Database,
) -> AppResult<Option<Snssai>> {
    let collection = db.collection::<SnssaiMapping>("snssai_mappings");

    let cursor = collection
        .find(doc! {
            "servingSnssai.sst": serving_snssai.sst as i32,
            "servingPlmnId.mcc": &serving_plmn.mcc,
            "servingPlmnId.mnc": &serving_plmn.mnc,
            "homePlmnId.mcc": &home_plmn.mcc,
            "homePlmnId.mnc": &home_plmn.mnc,
        })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query snssai_mappings: {}", e))
        })?;

    let mappings: Vec<SnssaiMapping> = cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read snssai_mappings: {}", e))
    })?;

    for mapping in &mappings {
        if !snssai_eq(serving_snssai, &mapping.serving_snssai) {
            continue;
        }

        match &mapping.validity_area {
            None => return Ok(Some(mapping.home_snssai.clone())),
            Some(tais) if tais.is_empty() => return Ok(Some(mapping.home_snssai.clone())),
            Some(tais) => {
                if tais.iter().any(|t| tai_eq(t, current_tai)) {
                    return Ok(Some(mapping.home_snssai.clone()));
                }
            }
        }
    }

    debug!(
        sst = serving_snssai.sst,
        sd = ?serving_snssai.sd,
        tac = %current_tai.tac,
        "No valid S-NSSAI mapping found for TAI"
    );

    Ok(None)
}
