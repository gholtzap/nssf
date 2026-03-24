use std::sync::Arc;

use bson::doc;
use futures::TryStreamExt;
use mongodb::Database;
use tracing::debug;

use crate::clients::NrfClient;
use crate::errors::{AppError, AppResult};
use crate::services::allowed_nssai::{check_subscription, evaluate_policy, tai_eq};
use crate::services::amf_selection::select_amf_candidates;
use crate::services::feature_negotiation::negotiate_features;
use crate::services::nsag_admission::{admit_to_nsag, check_nsag_admission};
use crate::services::nssrg_processing::assign_nssrg;
use crate::services::snssai_mapping::{map_snssai, map_snssai_with_tai};
use crate::services::subscription::{get_subscribed_snssais, get_subscription};
use crate::types::common::{PlmnId, Snssai, Tai};
use crate::types::db::{AccessType, NsiConfiguration, SliceConfiguration, UeSubscription};
use crate::types::nsselection::{
    AllowedNssai, AllowedSnssai, AuthorizedNetworkSliceInfo, ConfiguredSnssai, NsiInformation,
    RoamingIndication, SliceInfoForPDUSession, SliceInfoForRegistration,
    SliceInfoForUEConfigurationUpdate,
};
use crate::validation::snssai_eq;

fn plmn_eq(a: &PlmnId, b: &PlmnId) -> bool {
    a.mcc == b.mcc && a.mnc == b.mnc
}

fn empty_response() -> AuthorizedNetworkSliceInfo {
    AuthorizedNetworkSliceInfo {
        allowed_nssai_list: None,
        configured_nssai: None,
        target_amf_set: None,
        candidate_amf_list: None,
        rejected_nssai_in_plmn: None,
        rejected_nssai_in_ta: None,
        nsi_information: None,
        supported_features: None,
        nrf_amf_set: None,
        nrf_amf_set_nf_mgt_uri: None,
        nrf_amf_set_access_token_uri: None,
        nrf_oauth2_required: None,
        target_amf_service_set: None,
        target_nssai: None,
        nsag_infos: None,
        mapping_of_nssai: None,
    }
}

fn is_slice_available_in_tai(slice: &SliceConfiguration, tai: Option<&Tai>) -> bool {
    let tai = match tai {
        Some(t) => t,
        None => return true,
    };
    match &slice.tai_list {
        None => true,
        Some(tais) if tais.is_empty() => true,
        Some(tais) => tais.iter().any(|t| tai_eq(t, tai)),
    }
}

fn is_nsi_available_in_tai(nsi: &NsiConfiguration, tai: Option<&Tai>) -> bool {
    let tai = match tai {
        Some(t) => t,
        None => return true,
    };
    match &nsi.tai_list {
        None => true,
        Some(tais) if tais.is_empty() => true,
        Some(tais) => tais.iter().any(|t| tai_eq(t, tai)),
    }
}

fn opt_vec<T>(v: Vec<T>) -> Option<Vec<T>> {
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

async fn fetch_all_slices(db: &Database) -> AppResult<Vec<SliceConfiguration>> {
    let collection = db.collection::<SliceConfiguration>("slices");
    let cursor = collection
        .find(doc! {})
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query slices: {}", e)))?;
    cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read slices: {}", e)))
}

async fn select_nsi_for_snssai(
    snssai: &Snssai,
    plmn_id: &PlmnId,
    tai: Option<&Tai>,
    db: &Database,
) -> AppResult<Vec<NsiInformation>> {
    let collection = db.collection::<NsiConfiguration>("nsi");

    let mut filter = doc! {
        "snssai.sst": snssai.sst as i32,
        "plmnId.mcc": &plmn_id.mcc,
        "plmnId.mnc": &plmn_id.mnc,
    };
    if let Some(ref sd) = snssai.sd {
        filter.insert("snssai.sd", sd);
    }

    let cursor = collection
        .find(filter)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to query NSI: {}", e)))?;

    let nsi_configs: Vec<NsiConfiguration> = cursor
        .try_collect()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to read NSI: {}", e)))?;

    let mut available: Vec<&NsiConfiguration> = nsi_configs
        .iter()
        .filter(|nsi| snssai_eq(snssai, &nsi.snssai) && is_nsi_available_in_tai(nsi, tai))
        .collect();

    available.sort_by(|a, b| {
        let priority_cmp = b.priority.unwrap_or(0).cmp(&a.priority.unwrap_or(0));
        let load_cmp = a.load_level.unwrap_or(0).cmp(&b.load_level.unwrap_or(0));
        priority_cmp.then(load_cmp)
    });

    Ok(available
        .iter()
        .map(|nsi| NsiInformation {
            nrf_id: nsi.nrf_id.clone(),
            nsi_id: Some(nsi.nsi_id.clone()),
            nrf_nf_mgt_uri: nsi.nrf_nf_mgt_uri.clone(),
            nrf_access_token_uri: nsi.nrf_access_token_uri.clone(),
            nrf_oauth2_required: nsi.nrf_oauth2_required.clone(),
        })
        .collect())
}

async fn map_home_snssai(
    snssai: &Snssai,
    serving_plmn: &PlmnId,
    home_plmn: &PlmnId,
    tai: Option<&Tai>,
    db: &Database,
) -> AppResult<Option<Snssai>> {
    if let Some(t) = tai {
        map_snssai_with_tai(snssai, serving_plmn, home_plmn, t, db).await
    } else {
        map_snssai(snssai, serving_plmn, home_plmn, db).await
    }
}

async fn process_admission_and_nssrg(
    snssai: &Snssai,
    tai: Option<&Tai>,
    ue_sup_nssrg_ind: bool,
    suppress_nssrg_ind: bool,
    db: &Database,
) -> AppResult<Option<bool>> {
    let Some(t) = tai else {
        return Ok(Some(true));
    };

    if !check_nsag_admission(snssai, t, db).await? {
        return Ok(None);
    }

    let _ = admit_to_nsag(snssai, t, db).await?;

    if ue_sup_nssrg_ind && !suppress_nssrg_ind {
        let _ = assign_nssrg(snssai, t, db).await?;
    }

    Ok(Some(true))
}

async fn build_allowed_snssai_list(
    processed_snssais: &[Snssai],
    target_plmn_id: &PlmnId,
    serving_plmn_id: &PlmnId,
    home_plmn_id: &PlmnId,
    is_roaming: bool,
    tai: Option<&Tai>,
    db: &Database,
) -> AppResult<Vec<AllowedSnssai>> {
    let mut list = Vec::new();

    for snssai in processed_snssais {
        let nsi_information_list = select_nsi_for_snssai(snssai, target_plmn_id, tai, db).await?;

        let mut entry = AllowedSnssai {
            allowed_snssai: snssai.clone(),
            nsi_information_list: opt_vec(nsi_information_list),
            mapped_home_snssai: None,
        };

        if is_roaming {
            entry.mapped_home_snssai =
                map_home_snssai(snssai, serving_plmn_id, home_plmn_id, tai, db).await?;
        }

        list.push(entry);
    }

    Ok(list)
}

async fn build_configured_nssai(
    subscription: &UeSubscription,
    default_configured_snssai_ind: bool,
    available_slices: &[SliceConfiguration],
    target_plmn_id: &PlmnId,
    serving_plmn_id: &PlmnId,
    home_plmn_id: &PlmnId,
    is_roaming: bool,
    tai: Option<&Tai>,
    db: &Database,
) -> AppResult<Vec<ConfiguredSnssai>> {
    let mut configured = Vec::new();

    let snssais_for_configured: Vec<_> = if default_configured_snssai_ind {
        subscription
            .subscribed_snssais
            .iter()
            .filter(|s| s.default_indication == Some(true))
            .collect()
    } else {
        subscription.subscribed_snssais.iter().collect()
    };

    for subscribed in &snssais_for_configured {
        let slice = available_slices.iter().find(|s| {
            snssai_eq(&s.snssai, &subscribed.subscribed_snssai)
                && plmn_eq(&s.plmn_id, target_plmn_id)
        });

        if let Some(slice) = slice {
            if is_slice_available_in_tai(slice, tai) {
                let mut entry = ConfiguredSnssai {
                    configured_snssai: subscribed.subscribed_snssai.clone(),
                    mapped_home_snssai: None,
                };

                if is_roaming {
                    entry.mapped_home_snssai = map_home_snssai(
                        &subscribed.subscribed_snssai,
                        serving_plmn_id,
                        home_plmn_id,
                        tai,
                        db,
                    )
                    .await?;
                }

                configured.push(entry);
            }
        }
    }

    Ok(configured)
}

pub async fn select_for_registration(
    slice_info: &SliceInfoForRegistration,
    home_plmn_id: &PlmnId,
    supi: &str,
    tai: Option<&Tai>,
    supported_features: Option<&str>,
    db: &Database,
    nrf_client: Option<&Arc<NrfClient>>,
) -> AppResult<AuthorizedNetworkSliceInfo> {
    let subscription = get_subscription(db, supi, home_plmn_id).await?;

    let subscription = match subscription {
        Some(s) => s,
        None => {
            let requested = slice_info.requested_nssai.as_deref().unwrap_or_default();
            debug!(supi, "No subscription found for UE");
            return Ok(AuthorizedNetworkSliceInfo {
                rejected_nssai_in_plmn: opt_vec(requested.to_vec()),
                ..empty_response()
            });
        }
    };

    let serving_plmn_id = &subscription.plmn_id;
    let is_roaming = !plmn_eq(serving_plmn_id, home_plmn_id);
    let target_plmn_id = home_plmn_id;

    let available_slices = fetch_all_slices(db).await?;

    let requested_nssais = slice_info.requested_nssai.as_deref().unwrap_or_default();
    let default_configured_snssai_ind = slice_info.default_configured_snssai_ind.unwrap_or(false);
    let ue_sup_nssrg_ind = slice_info.ue_sup_nssrg_ind.unwrap_or(false);
    let suppress_nssrg_ind = slice_info.suppress_nssrg_ind.unwrap_or(false);

    let snssais_to_check: Vec<Snssai> =
        if default_configured_snssai_ind && requested_nssais.is_empty() {
            subscription
                .subscribed_snssais
                .iter()
                .filter(|s| s.default_indication == Some(true))
                .map(|s| s.subscribed_snssai.clone())
                .collect()
        } else if !requested_nssais.is_empty() {
            requested_nssais.to_vec()
        } else {
            get_subscribed_snssais(&subscription)
        };

    let mut processed_snssais: Vec<Snssai> = Vec::new();
    let mut rejected_nssai_in_plmn: Vec<Snssai> = Vec::new();
    let mut rejected_nssai_in_ta: Vec<Snssai> = Vec::new();

    for snssai in &snssais_to_check {
        if !check_subscription(snssai, &subscription) && !requested_nssais.is_empty() {
            rejected_nssai_in_plmn.push(snssai.clone());
            continue;
        }

        let available_slice = available_slices.iter().find(|s| {
            snssai_eq(&s.snssai, snssai) && plmn_eq(&s.plmn_id, target_plmn_id)
        });

        let available_slice = match available_slice {
            Some(s) => s,
            None => {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
        };

        if !is_slice_available_in_tai(available_slice, tai) {
            rejected_nssai_in_ta.push(snssai.clone());
            continue;
        }

        if let Some(t) = tai {
            if !evaluate_policy(snssai, target_plmn_id, t, db).await? {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
        }

        match process_admission_and_nssrg(snssai, tai, ue_sup_nssrg_ind, suppress_nssrg_ind, db)
            .await?
        {
            None => {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
            Some(_) => {}
        }

        processed_snssais.push(snssai.clone());
    }

    let mut allowed_nssai_list: Vec<AllowedNssai> = Vec::new();

    if !processed_snssais.is_empty() {
        let allowed_snssai_list = build_allowed_snssai_list(
            &processed_snssais,
            target_plmn_id,
            serving_plmn_id,
            home_plmn_id,
            is_roaming,
            tai,
            db,
        )
        .await?;

        allowed_nssai_list.push(AllowedNssai {
            allowed_snssai_list,
            access_type: AccessType::ThreeGppAccess,
        });
    }

    let configured_nssai = build_configured_nssai(
        &subscription,
        default_configured_snssai_ind,
        &available_slices,
        target_plmn_id,
        serving_plmn_id,
        home_plmn_id,
        is_roaming,
        tai,
        db,
    )
    .await?;

    let negotiated_features = negotiate_features(supported_features);

    let mut result = AuthorizedNetworkSliceInfo {
        allowed_nssai_list: opt_vec(allowed_nssai_list),
        configured_nssai: opt_vec(configured_nssai),
        rejected_nssai_in_plmn: opt_vec(rejected_nssai_in_plmn),
        rejected_nssai_in_ta: opt_vec(rejected_nssai_in_ta),
        supported_features: negotiated_features,
        ..empty_response()
    };

    if processed_snssais.is_empty() && !requested_nssais.is_empty() {
        let amf_result =
            select_amf_candidates(requested_nssais, target_plmn_id, db, nrf_client).await?;

        result.target_amf_set = amf_result.target_amf_set;
        result.target_amf_service_set = amf_result.target_amf_service_set;
        result.candidate_amf_list = opt_vec(amf_result.candidate_amf_list);
        result.nrf_amf_set = amf_result.nrf_amf_set;
        result.nrf_amf_set_nf_mgt_uri = amf_result.nrf_amf_set_nf_mgt_uri;
        result.nrf_amf_set_access_token_uri = amf_result.nrf_amf_set_access_token_uri;
        result.nrf_oauth2_required = amf_result.nrf_oauth2_required;
    }

    debug!(
        allowed = result.allowed_nssai_list.as_ref().map_or(0, |l| l.len()),
        rejected_plmn = result.rejected_nssai_in_plmn.as_ref().map_or(0, |l| l.len()),
        rejected_ta = result.rejected_nssai_in_ta.as_ref().map_or(0, |l| l.len()),
        "Registration slice selection complete"
    );

    Ok(result)
}

pub async fn select_for_pdu_session(
    slice_info: &SliceInfoForPDUSession,
    home_plmn_id: &PlmnId,
    supi: &str,
    tai: Option<&Tai>,
    supported_features: Option<&str>,
    db: &Database,
) -> AppResult<AuthorizedNetworkSliceInfo> {
    let requested_snssai = &slice_info.s_nssai;
    let roaming_indication = &slice_info.roaming_indication;
    let home_snssai = slice_info.home_snssai.as_ref();

    let subscription = get_subscription(db, supi, home_plmn_id).await?;

    let subscription = match subscription {
        Some(s) => s,
        None => {
            return Ok(AuthorizedNetworkSliceInfo {
                rejected_nssai_in_plmn: Some(vec![requested_snssai.clone()]),
                ..empty_response()
            });
        }
    };

    let serving_plmn_id = &subscription.plmn_id;
    let is_roaming = !plmn_eq(serving_plmn_id, home_plmn_id);

    let (target_snssai, target_plmn_id) = match roaming_indication {
        RoamingIndication::HomeRoutedRoaming if home_snssai.is_some() => {
            (home_snssai.unwrap(), home_plmn_id)
        }
        RoamingIndication::LocalBreakout if is_roaming => (requested_snssai, serving_plmn_id),
        _ => (requested_snssai, home_plmn_id),
    };

    if !check_subscription(target_snssai, &subscription) {
        return Ok(AuthorizedNetworkSliceInfo {
            rejected_nssai_in_plmn: Some(vec![requested_snssai.clone()]),
            ..empty_response()
        });
    }

    let available_slices = fetch_all_slices(db).await?;

    let available_slice = available_slices.iter().find(|s| {
        snssai_eq(&s.snssai, target_snssai) && plmn_eq(&s.plmn_id, target_plmn_id)
    });

    if available_slice.is_none() {
        return Ok(AuthorizedNetworkSliceInfo {
            rejected_nssai_in_plmn: Some(vec![requested_snssai.clone()]),
            ..empty_response()
        });
    }

    let available_slice = available_slice.unwrap();

    if !is_slice_available_in_tai(available_slice, tai) {
        return Ok(AuthorizedNetworkSliceInfo {
            rejected_nssai_in_ta: Some(vec![requested_snssai.clone()]),
            ..empty_response()
        });
    }

    if let Some(t) = tai {
        if !evaluate_policy(target_snssai, target_plmn_id, t, db).await? {
            return Ok(AuthorizedNetworkSliceInfo {
                rejected_nssai_in_plmn: Some(vec![requested_snssai.clone()]),
                ..empty_response()
            });
        }

        if !check_nsag_admission(target_snssai, t, db).await? {
            return Ok(AuthorizedNetworkSliceInfo {
                rejected_nssai_in_plmn: Some(vec![requested_snssai.clone()]),
                ..empty_response()
            });
        }

        let _ = admit_to_nsag(target_snssai, t, db).await?;
    }

    let nsi_information_list =
        select_nsi_for_snssai(requested_snssai, target_plmn_id, tai, db).await?;

    let mut allowed_snssai = AllowedSnssai {
        allowed_snssai: requested_snssai.clone(),
        nsi_information_list: opt_vec(nsi_information_list),
        mapped_home_snssai: None,
    };

    match roaming_indication {
        RoamingIndication::HomeRoutedRoaming if home_snssai.is_some() => {
            allowed_snssai.mapped_home_snssai = home_snssai.cloned();
        }
        RoamingIndication::LocalBreakout if is_roaming => {
            allowed_snssai.mapped_home_snssai = map_home_snssai(
                requested_snssai,
                serving_plmn_id,
                home_plmn_id,
                tai,
                db,
            )
            .await?;
        }
        _ => {}
    }

    let allowed_nssai = AllowedNssai {
        allowed_snssai_list: vec![allowed_snssai],
        access_type: AccessType::ThreeGppAccess,
    };

    let negotiated_features = negotiate_features(supported_features);

    debug!(
        sst = requested_snssai.sst,
        sd = ?requested_snssai.sd,
        "PDU session slice selection complete"
    );

    Ok(AuthorizedNetworkSliceInfo {
        allowed_nssai_list: Some(vec![allowed_nssai]),
        supported_features: negotiated_features,
        ..empty_response()
    })
}

pub async fn select_for_ue_cu(
    slice_info: &SliceInfoForUEConfigurationUpdate,
    home_plmn_id: &PlmnId,
    supi: &str,
    tai: Option<&Tai>,
    supported_features: Option<&str>,
    db: &Database,
) -> AppResult<AuthorizedNetworkSliceInfo> {
    let subscription = get_subscription(db, supi, home_plmn_id).await?;

    let subscription = match subscription {
        Some(s) => s,
        None => {
            let requested = slice_info.requested_nssai.as_deref().unwrap_or_default();
            return Ok(AuthorizedNetworkSliceInfo {
                rejected_nssai_in_plmn: opt_vec(requested.to_vec()),
                ..empty_response()
            });
        }
    };

    let serving_plmn_id = &subscription.plmn_id;
    let is_roaming = !plmn_eq(serving_plmn_id, home_plmn_id);
    let target_plmn_id = home_plmn_id;

    let available_slices = fetch_all_slices(db).await?;

    let requested_nssais = slice_info.requested_nssai.as_deref().unwrap_or_default();
    let rejected_nssai_ra = slice_info.rejected_nssai_ra.as_deref().unwrap_or_default();
    let default_configured_snssai_ind = slice_info.default_configured_snssai_ind.unwrap_or(false);
    let ue_sup_nssrg_ind = slice_info.ue_sup_nssrg_ind.unwrap_or(false);
    let suppress_nssrg_ind = slice_info.suppress_nssrg_ind.unwrap_or(false);

    let snssais_to_check: Vec<Snssai> =
        if default_configured_snssai_ind && requested_nssais.is_empty() {
            subscription
                .subscribed_snssais
                .iter()
                .filter(|s| s.default_indication == Some(true))
                .map(|s| s.subscribed_snssai.clone())
                .collect()
        } else if !requested_nssais.is_empty() {
            requested_nssais.to_vec()
        } else {
            get_subscribed_snssais(&subscription)
        };

    let mut processed_snssais: Vec<Snssai> = Vec::new();
    let mut rejected_nssai_in_plmn: Vec<Snssai> = Vec::new();
    let mut rejected_nssai_in_ta: Vec<Snssai> = Vec::new();

    for snssai in &snssais_to_check {
        if rejected_nssai_ra
            .iter()
            .any(|rejected| snssai_eq(rejected, snssai))
        {
            rejected_nssai_in_plmn.push(snssai.clone());
            continue;
        }

        if !check_subscription(snssai, &subscription) && !requested_nssais.is_empty() {
            rejected_nssai_in_plmn.push(snssai.clone());
            continue;
        }

        let available_slice = available_slices.iter().find(|s| {
            snssai_eq(&s.snssai, snssai) && plmn_eq(&s.plmn_id, target_plmn_id)
        });

        let available_slice = match available_slice {
            Some(s) => s,
            None => {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
        };

        if !is_slice_available_in_tai(available_slice, tai) {
            rejected_nssai_in_ta.push(snssai.clone());
            continue;
        }

        if let Some(t) = tai {
            if !evaluate_policy(snssai, target_plmn_id, t, db).await? {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
        }

        match process_admission_and_nssrg(snssai, tai, ue_sup_nssrg_ind, suppress_nssrg_ind, db)
            .await?
        {
            None => {
                rejected_nssai_in_plmn.push(snssai.clone());
                continue;
            }
            Some(_) => {}
        }

        processed_snssais.push(snssai.clone());
    }

    let mut allowed_nssai_list: Vec<AllowedNssai> = Vec::new();

    if !processed_snssais.is_empty() {
        let allowed_snssai_list = build_allowed_snssai_list(
            &processed_snssais,
            target_plmn_id,
            serving_plmn_id,
            home_plmn_id,
            is_roaming,
            tai,
            db,
        )
        .await?;

        allowed_nssai_list.push(AllowedNssai {
            allowed_snssai_list,
            access_type: AccessType::ThreeGppAccess,
        });
    }

    let configured_nssai = build_configured_nssai(
        &subscription,
        default_configured_snssai_ind,
        &available_slices,
        target_plmn_id,
        serving_plmn_id,
        home_plmn_id,
        is_roaming,
        tai,
        db,
    )
    .await?;

    let negotiated_features = negotiate_features(supported_features);

    debug!(
        allowed = allowed_nssai_list.len(),
        rejected_plmn = rejected_nssai_in_plmn.len(),
        rejected_ta = rejected_nssai_in_ta.len(),
        "UE configuration update slice selection complete"
    );

    Ok(AuthorizedNetworkSliceInfo {
        allowed_nssai_list: opt_vec(allowed_nssai_list),
        configured_nssai: opt_vec(configured_nssai),
        rejected_nssai_in_plmn: opt_vec(rejected_nssai_in_plmn),
        rejected_nssai_in_ta: opt_vec(rejected_nssai_in_ta),
        supported_features: negotiated_features,
        ..empty_response()
    })
}
