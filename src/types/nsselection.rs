use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::common::{NfType, PlmnId, Snssai, Tai};
use super::db::AccessType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoamingIndication {
    #[serde(rename = "NON_ROAMING")]
    NonRoaming,
    #[serde(rename = "LOCAL_BREAKOUT")]
    LocalBreakout,
    #[serde(rename = "HOME_ROUTED_ROAMING")]
    HomeRoutedRoaming,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsiInformation {
    pub nrf_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsi_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_nf_mgt_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_access_token_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_oauth2_required: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedSnssai {
    pub allowed_snssai: Snssai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsi_information_list: Option<Vec<NsiInformation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapped_home_snssai: Option<Snssai>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedNssai {
    pub allowed_snssai_list: Vec<AllowedSnssai>,
    pub access_type: AccessType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredSnssai {
    pub configured_snssai: Snssai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapped_home_snssai: Option<Snssai>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingOfSnssai {
    pub serving_snssai: Snssai,
    pub home_snssai: Snssai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribedSnssaiInfo {
    pub subscribed_snssai: Snssai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_indication: Option<bool>,
    #[serde(rename = "subscribedNsSrgList")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_ns_srg_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceInfoForRegistration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_nssai: Option<Vec<SubscribedSnssaiInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssai_current_access: Option<AllowedNssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssai_other_access: Option<AllowedNssai>,
    #[serde(rename = "sNssaiForMapping")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s_nssai_for_mapping: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_nssai: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_configured_snssai_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_of_nssai: Option<Vec<MappingOfSnssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_mapping: Option<bool>,
    #[serde(rename = "ueSupNssrgInd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ue_sup_nssrg_ind: Option<bool>,
    #[serde(rename = "suppressNssrgInd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_nssrg_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsag_supported: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceInfoForPDUSession {
    #[serde(rename = "sNssai")]
    pub s_nssai: Snssai,
    pub roaming_indication: RoamingIndication,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_snssai: Option<Snssai>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceInfoForUEConfigurationUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_nssai: Option<Vec<SubscribedSnssaiInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssai_current_access: Option<AllowedNssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssai_other_access: Option<AllowedNssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_configured_snssai_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_nssai: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_of_nssai: Option<Vec<MappingOfSnssai>>,
    #[serde(rename = "ueSupNssrgInd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ue_sup_nssrg_ind: Option<bool>,
    #[serde(rename = "suppressNssrgInd")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_nssrg_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_nssai_ra: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsag_supported: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TacRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiRange {
    pub plmn_id: PlmnId,
    pub tac_range_list: Vec<TacRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsagInfo {
    pub nsag_ids: Vec<u32>,
    pub snssai_list: Vec<Snssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizedNetworkSliceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssai_list: Option<Vec<AllowedNssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_nssai: Option<Vec<ConfiguredSnssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_amf_set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_amf_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_nssai_in_plmn: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_nssai_in_ta: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsi_information: Option<NsiInformation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_amf_set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_amf_set_nf_mgt_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_amf_set_access_token_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_oauth2_required: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_amf_service_set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_nssai: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsag_infos: Option<Vec<NsagInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping_of_nssai: Option<Vec<MappingOfSnssai>>,
}

#[derive(Debug, Deserialize)]
pub struct NsSelectionQueryParams {
    #[serde(rename = "nf-type")]
    pub nf_type: NfType,
    #[serde(rename = "nf-id")]
    pub nf_id: String,
    pub supi: Option<String>,
    #[serde(rename = "slice-info-request-for-registration")]
    pub slice_info_request_for_registration: Option<String>,
    #[serde(rename = "slice-info-request-for-pdu-session")]
    pub slice_info_request_for_pdu_session: Option<String>,
    #[serde(rename = "slice-info-request-for-ue-cu")]
    pub slice_info_request_for_ue_cu: Option<String>,
    #[serde(rename = "home-plmn-id")]
    pub home_plmn_id: Option<String>,
    pub tai: Option<String>,
    #[serde(rename = "supported-features")]
    pub supported_features: Option<String>,
}

impl NsSelectionQueryParams {
    pub fn parse_slice_info_for_registration(&self) -> Option<Result<SliceInfoForRegistration, serde_json::Error>> {
        self.slice_info_request_for_registration
            .as_ref()
            .map(|s| serde_json::from_str(s))
    }

    pub fn parse_slice_info_for_pdu_session(&self) -> Option<Result<SliceInfoForPDUSession, serde_json::Error>> {
        self.slice_info_request_for_pdu_session
            .as_ref()
            .map(|s| serde_json::from_str(s))
    }

    pub fn parse_slice_info_for_ue_cu(&self) -> Option<Result<SliceInfoForUEConfigurationUpdate, serde_json::Error>> {
        self.slice_info_request_for_ue_cu
            .as_ref()
            .map(|s| serde_json::from_str(s))
    }

    pub fn parse_home_plmn_id(&self) -> Option<Result<PlmnId, serde_json::Error>> {
        self.home_plmn_id
            .as_ref()
            .map(|s| serde_json::from_str(s))
    }

    pub fn parse_tai(&self) -> Option<Result<Tai, serde_json::Error>> {
        self.tai
            .as_ref()
            .map(|s| serde_json::from_str(s))
    }
}
