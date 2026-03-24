use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub server_host: String,
    pub mongodb_uri: String,
    pub mongodb_db_name: String,
    pub nrf_uri: Option<String>,
    pub home_plmn: String,
    pub allowed_plmns: Vec<String>,
    pub nf_instance_id: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?;

        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let mongodb_uri = env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let mongodb_db_name = env::var("MONGODB_DB_NAME")
            .unwrap_or_else(|_| "nssf".to_string());

        let nrf_uri = env::var("NRF_URI")
            .ok()
            .filter(|s| !s.is_empty());

        let home_plmn = env::var("HOME_PLMN")
            .unwrap_or_else(|_| "99970".to_string());

        let allowed_plmns = env::var("ALLOWED_PLMNS")
            .unwrap_or_else(|_| "99970".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let nf_instance_id = env::var("NF_INSTANCE_ID")
            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        Ok(Self {
            port,
            server_host,
            mongodb_uri,
            mongodb_db_name,
            nrf_uri,
            home_plmn,
            allowed_plmns,
            nf_instance_id,
        })
    }
}
