use mongodb::Database;
use std::sync::Arc;
use uuid::Uuid;

use crate::clients::NrfClient;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<Config>,
    pub nf_instance_id: Uuid,
    pub nrf_client: Option<Arc<NrfClient>>,
}

impl AppState {
    pub fn new(db: Database, config: Config, nrf_client: Option<Arc<NrfClient>>) -> Self {
        let nf_instance_id = config
            .nf_instance_id
            .parse::<Uuid>()
            .unwrap_or_else(|_| Uuid::new_v4());

        Self {
            db,
            config: Arc::new(config),
            nf_instance_id,
            nrf_client,
        }
    }
}
