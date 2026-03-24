mod clients;
mod config;
mod db;
mod errors;
mod handlers;
mod routes;
mod services;
mod types;
mod validation;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

use crate::clients::NrfClient;
use crate::config::Config;
use crate::types::AppState;
use crate::types::common::{NfStatus, NfType, PlmnId};
use crate::types::nrf::{NFProfile, NFService, NFServiceVersion, NfServiceStatus, PatchOp, PatchOperation};

fn parse_plmn(plmn_str: &str) -> PlmnId {
    if plmn_str.len() >= 5 {
        PlmnId {
            mcc: plmn_str[..3].to_string(),
            mnc: plmn_str[3..].to_string(),
        }
    } else {
        PlmnId {
            mcc: "999".to_string(),
            mnc: "70".to_string(),
        }
    }
}

fn build_nf_profile(config: &Config, nf_instance_id: &str) -> NFProfile {
    let plmn_list: Vec<PlmnId> = config
        .allowed_plmns
        .iter()
        .map(|p| parse_plmn(p))
        .collect();

    let services = vec![
        NFService {
            service_instance_id: uuid::Uuid::new_v4().to_string(),
            service_name: "nnssf-nsselection".to_string(),
            versions: vec![NFServiceVersion {
                api_version_in_uri: "v2".to_string(),
                api_full_version: "2.0.0".to_string(),
            }],
            scheme: "http".to_string(),
            nf_service_status: NfServiceStatus::Registered,
            fqdn: None,
            ipv4_addresses: Some(vec![config.server_host.clone()]),
            api_prefix: None,
            allowed_plmns: None,
            allowed_nf_types: Some(vec![NfType::Amf]),
            allowed_nssais: None,
            priority: None,
            capacity: None,
            load: None,
            supported_features: None,
        },
        NFService {
            service_instance_id: uuid::Uuid::new_v4().to_string(),
            service_name: "nnssf-nssaiavailability".to_string(),
            versions: vec![NFServiceVersion {
                api_version_in_uri: "v1".to_string(),
                api_full_version: "1.0.0".to_string(),
            }],
            scheme: "http".to_string(),
            nf_service_status: NfServiceStatus::Registered,
            fqdn: None,
            ipv4_addresses: Some(vec![config.server_host.clone()]),
            api_prefix: None,
            allowed_plmns: None,
            allowed_nf_types: Some(vec![NfType::Amf]),
            allowed_nssais: None,
            priority: None,
            capacity: None,
            load: None,
            supported_features: None,
        },
    ];

    NFProfile {
        nf_instance_id: nf_instance_id.to_string(),
        nf_type: NfType::Nssf,
        nf_status: NfStatus::Registered,
        plmn_list,
        s_nssai_list: None,
        nsi_list: None,
        fqdn: None,
        ipv4_addresses: Some(vec![config.server_host.clone()]),
        ipv6_addresses: None,
        allowed_plmns: None,
        allowed_nf_types: Some(vec![NfType::Amf]),
        priority: None,
        capacity: Some(100),
        load: Some(0),
        locality: None,
        nf_services: Some(services),
        amf_info: None,
        heart_beat_timer: Some(60),
    }
}

fn spawn_heartbeat_task(
    nrf_client: Arc<NrfClient>,
    nf_instance_id: String,
    heartbeat_timer: u32,
) {
    let interval_seconds = (heartbeat_timer as f64 * 0.8) as u64;
    tracing::info!(
        "Starting heartbeat task (interval: {}s, timer: {}s)",
        interval_seconds,
        heartbeat_timer
    );

    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
        interval.tick().await;

        loop {
            interval.tick().await;
            tracing::debug!("Sending heartbeat to NRF");

            let patch = vec![PatchOperation {
                op: PatchOp::Replace,
                path: "/nfStatus".to_string(),
                from: None,
                value: Some(serde_json::json!("REGISTERED")),
            }];

            match nrf_client.update_nf(&nf_instance_id, &patch).await {
                Ok(_) => {
                    tracing::debug!("Heartbeat sent successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to send heartbeat: {}", e);
                }
            }
        }
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nssf=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    tracing::info!("Connecting to MongoDB at {}", config.mongodb_uri);
    let mongo_client = mongodb::Client::with_uri_str(&config.mongodb_uri).await?;
    let db = mongo_client.database(&config.mongodb_db_name);
    db.run_command(bson::doc! { "ping": 1 }).await?;
    tracing::info!("MongoDB connected successfully");

    db::init_indexes(&db).await?;

    let nrf_client = config
        .nrf_uri
        .as_ref()
        .map(|uri| Arc::new(NrfClient::new(uri.clone())));

    let state = AppState::new(db, config.clone(), nrf_client.clone());
    let nf_instance_id = state.nf_instance_id.to_string();
    tracing::info!("NSSF instance ID: {}", nf_instance_id);

    if let Some(ref nrf) = nrf_client {
        let profile = build_nf_profile(&config, &nf_instance_id);
        tracing::info!("Registering NSSF with NRF at {}", nrf.base_url());

        match nrf.register_nf(&nf_instance_id, &profile).await {
            Ok(registered) => {
                tracing::info!("Successfully registered with NRF");
                let heartbeat_timer = registered.heart_beat_timer.unwrap_or(60);
                spawn_heartbeat_task(
                    Arc::clone(nrf),
                    nf_instance_id.clone(),
                    heartbeat_timer,
                );
            }
            Err(e) => {
                tracing::error!("Failed to register with NRF: {}", e);
                tracing::warn!("Continuing without NRF registration");
            }
        }
    } else {
        tracing::info!("NRF_URI not configured, skipping NRF registration");
    }

    let app = routes::create_routes(state);

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.port).parse()?;
    tracing::info!("NSSF server starting on http://{}:{}", config.server_host, config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let nrf_for_shutdown = nrf_client.clone();
    let nf_id_for_shutdown = nf_instance_id.clone();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("Shutdown signal received");

            if let Some(nrf) = nrf_for_shutdown {
                tracing::info!("Deregistering from NRF...");
                match nrf.deregister_nf(&nf_id_for_shutdown).await {
                    Ok(()) => tracing::info!("Successfully deregistered from NRF"),
                    Err(e) => tracing::error!("Failed to deregister from NRF: {}", e),
                }
            }
        })
        .await?;

    tracing::info!("NSSF shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
