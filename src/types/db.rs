use std::collections::HashMap;

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::{PlmnId, Snssai, Tai};
use super::nrf::Guami;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessType {
    #[serde(rename = "3GPP_ACCESS")]
    ThreeGppAccess,
    #[serde(rename = "NON_3GPP_ACCESS")]
    Non3GppAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SliceConfiguration {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub snssai: Snssai,
    pub plmn_id: PlmnId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_type: Option<AccessType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ue_support: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribedSnssai {
    pub subscribed_snssai: Snssai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_indication: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed_nssrg_list: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UeSubscription {
    pub supi: String,
    pub plmn_id: PlmnId,
    pub subscribed_snssais: Vec<SubscribedSnssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_snssai: Option<Snssai>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsiConfiguration {
    pub nsi_id: String,
    pub snssai: Snssai,
    pub plmn_id: PlmnId,
    pub nrf_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_nf_mgt_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_access_token_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_oauth2_required: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindow {
    pub start_time: String,
    pub end_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub days_of_week: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlicePolicy {
    pub policy_id: String,
    pub snssai: Snssai,
    pub plmn_id: PlmnId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ues_per_slice: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sessions_per_ue: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_time_windows: Option<Vec<TimeWindow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_time_windows: Option<Vec<TimeWindow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_priority_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_load_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_subscription_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub denied_tai_list: Option<Vec<Tai>>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnssaiMapping {
    pub mapping_id: String,
    pub serving_plmn_id: PlmnId,
    pub home_plmn_id: PlmnId,
    pub serving_snssai: Snssai,
    pub home_snssai: Snssai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validity_area: Option<Vec<Tai>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaiRange {
    pub start: String,
    pub end: String,
    pub plmn_id: PlmnId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsagConfiguration {
    pub nsag_id: u32,
    pub snssai_list: Vec<Snssai>,
    pub plmn_id: PlmnId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ue_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ue_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssrgConfiguration {
    pub nssrg_id: String,
    pub snssai_list: Vec<Snssai>,
    pub plmn_id: PlmnId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_list: Option<Vec<Tai>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tai_range_list: Option<Vec<TaiRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_ue_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_ue_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfSetConfig {
    pub amf_set_id: String,
    pub plmn_id: PlmnId,
    pub supported_snssais: Vec<Snssai>,
    pub nrf_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_nf_mgt_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_access_token_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nrf_oauth2_required: Option<HashMap<String, bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfServiceSetConfig {
    pub amf_service_set_id: String,
    pub amf_set_id: String,
    pub plmn_id: PlmnId,
    pub supported_snssais: Vec<Snssai>,
    pub nrf_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmfInstanceConfig {
    pub nf_instance_id: String,
    pub amf_set_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amf_service_set_id: Option<String>,
    pub plmn_id: PlmnId,
    pub supported_snssais: Vec<Snssai>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guami: Option<Guami>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssaiAvailabilitySubscriptionData {
    pub tai: Tai,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_snssai_list: Option<Vec<Snssai>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NssaiAvailabilitySubscription {
    pub subscription_id: String,
    pub nf_instance_id: String,
    pub subscription_data: NssaiAvailabilitySubscriptionData,
    pub notification_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_time: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
