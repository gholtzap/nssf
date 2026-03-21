mod clients;
mod config;
mod db;
mod errors;
mod types;

use axum::{Router, routing::get};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::signal;

use crate::clients::NrfClient;
use crate::config::Config;
use crate::types::AppState;
use crate::types::common::{NfStatus, NfType, PlmnId, Snssai};
use crate::types::nrf::{
    NFProfile, NFService, NFServiceVersion, NfServiceStatus, PatchOperation, PatchOp,
};

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

    let nrf_client = config.nrf_uri.as_ref().map(|uri| {
        tracing::info!("NRF URI configured: {}", uri);
        Arc::new(NrfClient::new(uri.clone()))
    });

    let state = AppState::new(db, config.clone(), nrf_client.clone());
    tracing::info!("NSSF instance ID: {}", state.nf_instance_id);

    let nf_instance_id = state.nf_instance_id;

    let home_plmn_str = &config.home_plmn;
    let mcc = home_plmn_str[..3].to_string();
    let mnc = home_plmn_str[3..].to_string();
    let home_plmn = PlmnId { mcc, mnc };

    if let Some(ref nrf) = nrf_client {
        let nf_profile = build_nf_profile(
            nf_instance_id.to_string(),
            home_plmn.clone(),
            config.server_host.clone(),
            config.port,
        );

        tracing::info!("Registering NSSF with NRF (instance ID: {})", nf_instance_id);
        match nrf.register_nf(&nf_instance_id.to_string(), &nf_profile).await {
            Ok(registered) => {
                tracing::info!("Successfully registered with NRF");

                let heartbeat_timer = registered.heart_beat_timer.unwrap_or(60);
                tracing::info!("Heartbeat timer: {} seconds", heartbeat_timer);

                let nrf_hb = Arc::clone(nrf);
                let nf_id = nf_instance_id.to_string();
                tokio::spawn(async move {
                    let interval_secs = (heartbeat_timer as f64 * 0.8) as u64;
                    let mut interval =
                        tokio::time::interval(std::time::Duration::from_secs(interval_secs));
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

                        match nrf_hb.update_nf(&nf_id, &patch).await {
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
            Err(e) => {
                tracing::error!("Failed to register with NRF: {}", e);
                tracing::warn!("Continuing without NRF registration");
            }
        }
    } else {
        tracing::warn!("NRF_URI not configured, skipping NRF registration");
    }

    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.port).parse()?;
    tracing::info!("NSSF server starting on http://{}:{}", config.server_host, config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let nrf_shutdown = nrf_client.clone();
    let shutdown_nf_id = nf_instance_id.to_string();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("Shutdown signal received, deregistering from NRF...");
            if let Some(ref nrf) = nrf_shutdown {
                match nrf.deregister_nf(&shutdown_nf_id).await {
                    Ok(_) => tracing::info!("Successfully deregistered from NRF"),
                    Err(e) => tracing::error!("Failed to deregister from NRF: {}", e),
                }
            }
        })
        .await?;

    tracing::info!("NSSF shutdown complete");
    Ok(())
}

fn build_nf_profile(
    nf_instance_id: String,
    home_plmn: PlmnId,
    host: String,
    port: u16,
) -> NFProfile {
    let api_prefix = format!("http://{}:{}", host, port);

    let nsselection_service = NFService {
        service_instance_id: uuid::Uuid::new_v4().to_string(),
        service_name: "nnssf-nsselection".to_string(),
        versions: vec![NFServiceVersion {
            api_version_in_uri: "v2".to_string(),
            api_full_version: "2.0.0".to_string(),
        }],
        scheme: "http".to_string(),
        nf_service_status: NfServiceStatus::Registered,
        fqdn: None,
        ipv4_addresses: Some(vec![host.clone()]),
        api_prefix: Some(api_prefix.clone()),
        allowed_plmns: None,
        allowed_nf_types: Some(vec![NfType::Amf]),
        allowed_nssais: None,
        priority: Some(0),
        capacity: Some(100),
        load: Some(0),
        supported_features: None,
    };

    let nssaiavailability_service = NFService {
        service_instance_id: uuid::Uuid::new_v4().to_string(),
        service_name: "nnssf-nssaiavailability".to_string(),
        versions: vec![NFServiceVersion {
            api_version_in_uri: "v1".to_string(),
            api_full_version: "1.0.0".to_string(),
        }],
        scheme: "http".to_string(),
        nf_service_status: NfServiceStatus::Registered,
        fqdn: None,
        ipv4_addresses: Some(vec![host.clone()]),
        api_prefix: Some(api_prefix),
        allowed_plmns: None,
        allowed_nf_types: Some(vec![NfType::Amf]),
        allowed_nssais: None,
        priority: Some(0),
        capacity: Some(100),
        load: Some(0),
        supported_features: None,
    };

    let default_snssai = Snssai {
        sst: 1,
        sd: Some("000001".to_string()),
    };

    NFProfile {
        nf_instance_id,
        nf_type: NfType::Nssf,
        nf_status: NfStatus::Registered,
        plmn_list: vec![home_plmn],
        s_nssai_list: Some(vec![default_snssai]),
        nsi_list: None,
        fqdn: None,
        ipv4_addresses: Some(vec![host]),
        ipv6_addresses: None,
        allowed_plmns: None,
        allowed_nf_types: Some(vec![NfType::Amf]),
        priority: Some(0),
        capacity: Some(100),
        load: Some(0),
        locality: None,
        nf_services: Some(vec![nsselection_service, nssaiavailability_service]),
        amf_info: None,
        heart_beat_timer: Some(60),
    }
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

async fn health_check() -> &'static str {
    "OK"
}
