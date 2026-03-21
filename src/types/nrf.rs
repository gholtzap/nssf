use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::common::{NfStatus, NfType, PlmnId, Snssai};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFProfile {
    pub nf_instance_id: String,
    pub nf_type: NfType,
    pub nf_status: NfStatus,
    pub plmn_list: Vec<PlmnId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s_nssai_list: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsi_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_plmns: Option<Vec<PlmnId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nf_types: Option<Vec<NfType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_services: Option<Vec<NFService>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_info: Option<AmfInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_beat_timer: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFService {
    pub service_instance_id: String,
    pub service_name: String,
    pub versions: Vec<NFServiceVersion>,
    pub scheme: String,
    pub nf_service_status: NfServiceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_plmns: Option<Vec<PlmnId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nf_types: Option<Vec<NfType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nssais: Option<Vec<Snssai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFServiceVersion {
    pub api_version_in_uri: String,
    pub api_full_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfServiceStatus {
    Registered,
    Suspended,
    Undiscoverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFRegisterRequest {
    #[serde(flatten)]
    pub nf_profile: NFProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFRegisterResponse {
    #[serde(flatten)]
    pub nf_profile: NFProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NfInstanceInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_disc_api_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_search: Option<PreferredSearch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_altered_priorities: Option<HashMap<String, u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferredSearch {
    #[serde(default)]
    pub preferred_tai_match_ind: bool,
    #[serde(default)]
    pub preferred_full_plmn_match_ind: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_api_versions_match_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_api_versions_ind: Option<bool>,
    #[serde(default)]
    pub preferred_locality_match_ind: bool,
    #[serde(default)]
    pub other_locality_ind: bool,
    #[serde(default)]
    pub preferred_vendor_specific_features_ind: bool,
    #[serde(default)]
    pub preferred_collocated_nf_type_ind: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_pgw_match_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_analytics_delays_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_features_match_ind: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_preferred_features_ind: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_set_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_region_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guami_list: Option<Vec<Guami>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<super::common::Tai>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Guami {
    pub plmn_id: PlmnId,
    pub amf_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_period: Option<u32>,
    #[serde(default)]
    pub nf_instances: Vec<NFProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_nf_inst_complete: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchOperation {
    pub op: PatchOp,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchOp {
    Add,
    Remove,
    Replace,
    Move,
    Copy,
    Test,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceName {
    NnssfNsselection,
    NnssfNssaiavailability,
}

impl ServiceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceName::NnssfNsselection => "nnssf-nsselection",
            ServiceName::NnssfNssaiavailability => "nnssf-nssaiavailability",
        }
    }
}

impl std::fmt::Display for ServiceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Serialize for ServiceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ServiceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "nnssf-nsselection" => Ok(ServiceName::NnssfNsselection),
            "nnssf-nssaiavailability" => Ok(ServiceName::NnssfNssaiavailability),
            _ => Err(serde::de::Error::unknown_variant(&s, &["nnssf-nsselection", "nnssf-nssaiavailability"])),
        }
    }
}
