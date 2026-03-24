use std::collections::HashMap;
use std::sync::Arc;

use futures::TryStreamExt;
use mongodb::Database;
use tracing::{debug, warn};

use crate::clients::NrfClient;
use crate::errors::{AppError, AppResult};
use crate::types::common::{NfStatus, NfType, PlmnId, Snssai};
use crate::types::db::{AmfInstanceConfig, AmfServiceSetConfig, AmfSetConfig};
use crate::types::nrf::NFProfile;
use crate::validation::snssai_eq;

#[derive(Debug, Clone, Default)]
pub struct AmfSelectionResult {
    pub target_amf_set: Option<String>,
    pub candidate_amf_list: Vec<String>,
    pub nrf_amf_set: Option<String>,
    pub nrf_amf_set_nf_mgt_uri: Option<String>,
    pub nrf_amf_set_access_token_uri: Option<String>,
    pub nrf_oauth2_required: Option<HashMap<String, bool>>,
    pub target_amf_service_set: Option<String>,
}

fn supports_all_snssais(supported: &[Snssai], required: &[Snssai]) -> bool {
    required
        .iter()
        .all(|req| supported.iter().any(|sup| snssai_eq(sup, req)))
}

fn plmn_eq(a: &PlmnId, b: &PlmnId) -> bool {
    a.mcc == b.mcc && a.mnc == b.mnc
}

async fn find_amf_sets(
    plmn: &PlmnId,
    db: &Database,
) -> AppResult<Vec<AmfSetConfig>> {
    let collection = db.collection::<AmfSetConfig>("amf_sets");

    let cursor = collection
        .find(bson::doc! {
            "plmnId.mcc": &plmn.mcc,
            "plmnId.mnc": &plmn.mnc,
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query AMF sets: {}", e)))?;

    cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read AMF sets: {}", e)))
}

fn select_target_amf_set<'a>(
    amf_sets: &'a [AmfSetConfig],
    required_snssais: &[Snssai],
) -> Option<&'a AmfSetConfig> {
    let mut matching: Vec<&AmfSetConfig> = amf_sets
        .iter()
        .filter(|s| supports_all_snssais(&s.supported_snssais, required_snssais))
        .collect();

    if matching.is_empty() {
        return None;
    }

    matching.sort_by(|a, b| {
        let priority_cmp = b
            .priority
            .unwrap_or(0)
            .cmp(&a.priority.unwrap_or(0));
        let capacity_cmp = b
            .capacity
            .unwrap_or(0)
            .cmp(&a.capacity.unwrap_or(0));
        priority_cmp.then(capacity_cmp)
    });

    Some(matching[0])
}

async fn find_amf_service_sets(
    amf_set_id: &str,
    db: &Database,
) -> AppResult<Vec<AmfServiceSetConfig>> {
    let collection = db.collection::<AmfServiceSetConfig>("amf_service_sets");

    let cursor = collection
        .find(bson::doc! { "amfSetId": amf_set_id })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query AMF service sets: {}", e))
        })?;

    cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read AMF service sets: {}", e))
    })
}

fn select_target_amf_service_set<'a>(
    service_sets: &'a [AmfServiceSetConfig],
    required_snssais: &[Snssai],
) -> Option<&'a AmfServiceSetConfig> {
    let mut matching: Vec<&AmfServiceSetConfig> = service_sets
        .iter()
        .filter(|ss| supports_all_snssais(&ss.supported_snssais, required_snssais))
        .collect();

    if matching.is_empty() {
        return None;
    }

    matching.sort_by(|a, b| b.priority.unwrap_or(0).cmp(&a.priority.unwrap_or(0)));

    Some(matching[0])
}

async fn find_amf_instances(
    amf_set_id: &str,
    plmn: &PlmnId,
    db: &Database,
) -> AppResult<Vec<AmfInstanceConfig>> {
    let collection = db.collection::<AmfInstanceConfig>("amf_instances");

    let cursor = collection
        .find(bson::doc! {
            "amfSetId": amf_set_id,
            "plmnId.mcc": &plmn.mcc,
            "plmnId.mnc": &plmn.mnc,
        })
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to query AMF instances: {}", e))
        })?;

    cursor.try_collect().await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read AMF instances: {}", e))
    })
}

fn generate_candidate_amf_list(
    instances: &[AmfInstanceConfig],
    required_snssais: &[Snssai],
) -> Vec<String> {
    let mut matching: Vec<&AmfInstanceConfig> = instances
        .iter()
        .filter(|i| supports_all_snssais(&i.supported_snssais, required_snssais))
        .collect();

    matching.sort_by(|a, b| {
        let capacity_cmp = b
            .capacity
            .unwrap_or(0)
            .cmp(&a.capacity.unwrap_or(0));
        let load_cmp = a
            .load_level
            .unwrap_or(0)
            .cmp(&b.load_level.unwrap_or(0));
        capacity_cmp.then(load_cmp)
    });

    matching
        .iter()
        .map(|i| i.nf_instance_id.clone())
        .collect()
}

async fn perform_local_amf_selection(
    snssais: &[Snssai],
    plmn: &PlmnId,
    db: &Database,
) -> AppResult<AmfSelectionResult> {
    let amf_sets = find_amf_sets(plmn, db).await?;

    let amf_set = match select_target_amf_set(&amf_sets, snssais) {
        Some(set) => set,
        None => {
            debug!("No matching AMF set found for requested S-NSSAIs");
            return Ok(AmfSelectionResult::default());
        }
    };

    let mut result = AmfSelectionResult {
        target_amf_set: Some(amf_set.amf_set_id.clone()),
        nrf_amf_set: Some(amf_set.nrf_id.clone()),
        nrf_amf_set_nf_mgt_uri: amf_set.nrf_nf_mgt_uri.clone(),
        nrf_amf_set_access_token_uri: amf_set.nrf_access_token_uri.clone(),
        nrf_oauth2_required: amf_set.nrf_oauth2_required.clone(),
        ..Default::default()
    };

    let service_sets = find_amf_service_sets(&amf_set.amf_set_id, db).await?;
    if let Some(ss) = select_target_amf_service_set(&service_sets, snssais) {
        result.target_amf_service_set = Some(ss.amf_service_set_id.clone());
    }

    let instances = find_amf_instances(&amf_set.amf_set_id, plmn, db).await?;
    result.candidate_amf_list = generate_candidate_amf_list(&instances, snssais);

    debug!(
        amf_set = ?result.target_amf_set,
        candidates = result.candidate_amf_list.len(),
        "Local AMF selection complete"
    );

    Ok(result)
}

fn rank_nrf_amf_profiles(profiles: &mut [NFProfile]) {
    profiles.sort_by(|a, b| {
        let priority_cmp = b
            .priority
            .unwrap_or(0)
            .cmp(&a.priority.unwrap_or(0));
        let capacity_cmp = b
            .capacity
            .unwrap_or(0)
            .cmp(&a.capacity.unwrap_or(0));
        let load_cmp = a.load.unwrap_or(0).cmp(&b.load.unwrap_or(0));
        priority_cmp.then(capacity_cmp).then(load_cmp)
    });
}

async fn discover_amf_via_nrf(
    snssais: &[Snssai],
    plmn: &PlmnId,
    nrf_client: &NrfClient,
) -> AppResult<AmfSelectionResult> {
    let mut query_params = HashMap::new();
    query_params.insert(
        "requester-nf-type".to_string(),
        "NSSF".to_string(),
    );

    let snssais_json = serde_json::to_string(snssais).unwrap_or_default();
    query_params.insert("snssais".to_string(), snssais_json);

    let plmn_json = serde_json::to_string(&[plmn]).unwrap_or_default();
    query_params.insert("target-plmn-list".to_string(), plmn_json);

    let search_result = nrf_client
        .discover_nf(NfType::Amf, Some(query_params))
        .await
        .map_err(|e| {
            warn!(error = %e, "NRF AMF discovery failed");
            AppError::ServiceUnavailable(format!("NRF AMF discovery failed: {}", e))
        })?;

    let mut active_profiles: Vec<NFProfile> = search_result
        .nf_instances
        .into_iter()
        .filter(|p| p.nf_status == NfStatus::Registered)
        .collect();

    if active_profiles.is_empty() {
        debug!("No active AMF instances discovered from NRF");
        return Ok(AmfSelectionResult::default());
    }

    rank_nrf_amf_profiles(&mut active_profiles);

    let candidate_list: Vec<String> = active_profiles
        .iter()
        .map(|p| p.nf_instance_id.clone())
        .collect();

    debug!(
        candidates = candidate_list.len(),
        "NRF AMF discovery complete"
    );

    Ok(AmfSelectionResult {
        candidate_amf_list: candidate_list,
        ..Default::default()
    })
}

pub async fn select_amf_candidates(
    snssais: &[Snssai],
    plmn: &PlmnId,
    db: &Database,
    nrf_client: Option<&Arc<NrfClient>>,
) -> AppResult<AmfSelectionResult> {
    let local_result = perform_local_amf_selection(snssais, plmn, db).await?;

    if !local_result.candidate_amf_list.is_empty() {
        return Ok(local_result);
    }

    if local_result.target_amf_set.is_some() {
        return Ok(local_result);
    }

    if let Some(client) = nrf_client {
        match discover_amf_via_nrf(snssais, plmn, client).await {
            Ok(nrf_result) if !nrf_result.candidate_amf_list.is_empty() => {
                return Ok(nrf_result);
            }
            Ok(_) => {
                debug!("NRF discovery returned no candidates, using local result");
            }
            Err(e) => {
                warn!(error = %e, "NRF fallback failed, using local result");
            }
        }
    }

    Ok(local_result)
}
