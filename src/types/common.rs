use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlmnId {
    pub mcc: String,
    pub mnc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snssai {
    pub sst: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Tai {
    pub plmn_id: PlmnId,
    pub tac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfType {
    Nrf,
    Udm,
    Amf,
    Smf,
    Ausf,
    Nef,
    Pcf,
    Smsf,
    Nssf,
    Udr,
    Lmf,
    Gmlc,
    #[serde(rename = "5G_EIR")]
    FiveGEir,
    Sepp,
    Upf,
    N3Iwf,
    Af,
    Udsf,
    Bsf,
    Chf,
    Nwdaf,
    Pcscf,
    Cbcf,
    Hss,
    Ucmf,
    SorAf,
    SprAf,
    Mme,
    Scsas,
    Scef,
    Scp,
    Nssaaf,
    Icscf,
    Scscf,
    Dra,
    ImsAs,
    Aanf,
    #[serde(rename = "5G_DDNMF")]
    FiveGDdnmf,
    Nsacf,
    Mfaf,
    Easdf,
    Dccf,
    MbSmf,
    Tsctsf,
    Adrf,
    GbaBsf,
    Cef,
    MbUpf,
    Nswof,
    Pkmf,
    Mnpf,
    SmsGmsc,
    SmsIwmsc,
    Mbsf,
    Mbstf,
    Panf,
}

impl std::fmt::Display for NfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NfType::FiveGEir => write!(f, "5G_EIR"),
            NfType::FiveGDdnmf => write!(f, "5G_DDNMF"),
            other => {
                let s = serde_json::to_string(other).unwrap_or_default();
                write!(f, "{}", s.trim_matches('"'))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfStatus {
    Registered,
    Suspended,
    Undiscoverable,
}

impl std::fmt::Display for NfStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NfStatus::Registered => write!(f, "REGISTERED"),
            NfStatus::Suspended => write!(f, "SUSPENDED"),
            NfStatus::Undiscoverable => write!(f, "UNDISCOVERABLE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_params: Option<Vec<InvalidParam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidParam {
    pub param: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl ProblemDetails {
    pub fn new(status: u16, title: &str, detail: &str) -> Self {
        let problem_type = match status {
            400 => Some("https://tools.ietf.org/html/rfc7231#section-6.5.1".to_string()),
            404 => Some("https://tools.ietf.org/html/rfc7231#section-6.5.4".to_string()),
            500 => Some("https://tools.ietf.org/html/rfc7231#section-6.6.1".to_string()),
            503 => Some("https://tools.ietf.org/html/rfc7231#section-6.6.3".to_string()),
            504 => Some("https://tools.ietf.org/html/rfc7231#section-6.6.5".to_string()),
            _ => None,
        };

        Self {
            r#type: problem_type,
            title: Some(title.to_string()),
            status: Some(status),
            detail: Some(detail.to_string()),
            instance: None,
            cause: None,
            invalid_params: None,
            supported_features: None,
        }
    }

    pub fn with_cause(mut self, cause: &str) -> Self {
        self.cause = Some(cause.to_string());
        self
    }

    pub fn with_instance(mut self, instance: &str) -> Self {
        self.instance = Some(instance.to_string());
        self
    }

    pub fn with_invalid_params(mut self, params: Vec<InvalidParam>) -> Self {
        self.invalid_params = Some(params);
        self
    }
}
