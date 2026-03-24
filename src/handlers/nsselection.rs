use axum::{
    extract::{Query, State},
    Json,
};
use tracing::{debug, warn};

use crate::errors::{AppError, AppResult};
use crate::services::network_slice_selection::{
    select_for_pdu_session, select_for_registration, select_for_ue_cu,
};
use crate::types::common::PlmnId;
use crate::types::nsselection::{AuthorizedNetworkSliceInfo, NsSelectionQueryParams};
use crate::types::AppState;
use crate::validation::validate_supi;

fn parse_home_plmn_from_config(home_plmn: &str) -> PlmnId {
    if home_plmn.len() >= 5 {
        PlmnId {
            mcc: home_plmn[..3].to_string(),
            mnc: home_plmn[3..].to_string(),
        }
    } else {
        PlmnId {
            mcc: "999".to_string(),
            mnc: "70".to_string(),
        }
    }
}

pub async fn get_network_slice_information(
    State(state): State<AppState>,
    Query(params): Query<NsSelectionQueryParams>,
) -> AppResult<Json<AuthorizedNetworkSliceInfo>> {
    debug!(
        nf_type = ?params.nf_type,
        nf_id = %params.nf_id,
        supi = ?params.supi,
        "NS selection request received"
    );

    let home_plmn_id = match params.parse_home_plmn_id() {
        Some(Ok(plmn)) => plmn,
        Some(Err(e)) => {
            return Err(AppError::BadRequest(format!(
                "Invalid home-plmn-id: {}",
                e
            )));
        }
        None => {
            if let Some(ref supi) = params.supi {
                if let Some((_, plmn)) = validate_supi(supi) {
                    plmn
                } else {
                    parse_home_plmn_from_config(&state.config.home_plmn)
                }
            } else {
                parse_home_plmn_from_config(&state.config.home_plmn)
            }
        }
    };

    let tai = match params.parse_tai() {
        Some(Ok(tai)) => Some(tai),
        Some(Err(e)) => {
            return Err(AppError::BadRequest(format!("Invalid tai: {}", e)));
        }
        None => None,
    };

    let supi = params.supi.as_deref().unwrap_or("");
    let supported_features = params.supported_features.as_deref();

    if let Some(result) = params.parse_slice_info_for_registration() {
        let slice_info = result
            .map_err(|e| AppError::BadRequest(format!("Invalid slice-info-request-for-registration: {}", e)))?;

        debug!("Dispatching to registration slice selection");
        let info = select_for_registration(
            &slice_info,
            &home_plmn_id,
            supi,
            tai.as_ref(),
            supported_features,
            &state.db,
            state.nrf_client.as_ref(),
        )
        .await?;
        return Ok(Json(info));
    }

    if let Some(result) = params.parse_slice_info_for_pdu_session() {
        let slice_info = result
            .map_err(|e| AppError::BadRequest(format!("Invalid slice-info-request-for-pdu-session: {}", e)))?;

        debug!("Dispatching to PDU session slice selection");
        let info = select_for_pdu_session(
            &slice_info,
            &home_plmn_id,
            supi,
            tai.as_ref(),
            supported_features,
            &state.db,
        )
        .await?;
        return Ok(Json(info));
    }

    if let Some(result) = params.parse_slice_info_for_ue_cu() {
        let slice_info = result
            .map_err(|e| AppError::BadRequest(format!("Invalid slice-info-request-for-ue-cu: {}", e)))?;

        debug!("Dispatching to UE configuration update slice selection");
        let info = select_for_ue_cu(
            &slice_info,
            &home_plmn_id,
            supi,
            tai.as_ref(),
            supported_features,
            &state.db,
        )
        .await?;
        return Ok(Json(info));
    }

    warn!("NS selection request missing slice info parameter");
    Err(AppError::BadRequest(
        "One of slice-info-request-for-registration, slice-info-request-for-pdu-session, or slice-info-request-for-ue-cu must be provided".to_string(),
    ))
}
