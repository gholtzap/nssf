use serde::{Deserialize, Serialize};

use super::common::{PlmnId, Snssai, Tai};
use super::nsselection::TaiRange;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NssfEventType {
    #[serde(rename = "SNSSAI_STATUS_CHANGE_REPORT")]
    SnssaiStatusChangeReport,
    #[serde(rename = "SNSSAI_REPLACEMENT_REPORT")]
    SnssaiReplacementReport,
    #[serde(rename = "NSI_UNAVAILABILITY_REPORT")]
    NsiUnavailabilityReport,
    #[serde(rename = "SNSSAI_VALIDITY_TIME_REPORT")]
    SnssaiValidityTimeReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestrictedSnssai {
    pub home_plmn_id: PlmnId,
    #[serde(rename = "sNssaiList")]
    pub s_nssai_list: Vec<Snssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_plmn_snssai_list: Option<Vec<Snssai>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedNssaiAvailabilityData {
    pub tai: Tai,
    pub supported_snssai_list: Vec<Snssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizedNssaiAvailabilityData {
    pub tai: Tai,
    pub supported_snssai_list: Vec<Snssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_snssai_list: Option<Vec<RestrictedSnssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssaiAvailabilityInfo {
    pub supported_nssai_availability_data: Vec<SupportedNssaiAvailabilityData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizedNssaiAvailabilityInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_nssai_availability_data: Option<Vec<AuthorizedNssaiAvailabilityData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssfEventSubscriptionCreateData {
    pub nf_nssai_availability_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    pub event: NssfEventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_events: Option<Vec<NssfEventType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_amf_set_tai_ind: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssfEventSubscriptionCreatedData {
    pub subscription_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_nssai_availability_data: Option<Vec<AuthorizedNssaiAvailabilityData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssfEventNotification {
    pub subscription_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_nssai_availability_data: Option<Vec<AuthorizedNssaiAvailabilityData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchItem {
    pub op: PatchOp,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PatchOp {
    Add,
    Remove,
    Replace,
    Move,
    Copy,
    Test,
}
