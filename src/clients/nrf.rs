use crate::types::nrf::{NFProfile, PatchOperation, SearchResult};
use crate::types::NfType;
use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use std::collections::HashMap;

pub struct NrfClient {
    client: Client,
    base_url: String,
}

impl NrfClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url,
        }
    }

    pub async fn register_nf(
        &self,
        nf_instance_id: &str,
        profile: &NFProfile,
    ) -> Result<NFProfile> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.base_url, nf_instance_id
        );

        let response = self
            .client
            .put(&url)
            .json(profile)
            .send()
            .await
            .context("Failed to send registration request to NRF")?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let registered_profile: NFProfile = response
                    .json()
                    .await
                    .context("Failed to parse NRF registration response")?;

                tracing::info!(
                    "Successfully registered NF instance {} with NRF",
                    nf_instance_id
                );

                Ok(registered_profile)
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF registration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn update_nf(
        &self,
        nf_instance_id: &str,
        patch_operations: &[PatchOperation],
    ) -> Result<Option<NFProfile>> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.base_url, nf_instance_id
        );

        let response = self
            .client
            .patch(&url)
            .json(patch_operations)
            .header("Content-Type", "application/json-patch+json")
            .send()
            .await
            .context("Failed to send update request to NRF")?;

        match response.status() {
            StatusCode::OK => {
                let profile: NFProfile = response
                    .json()
                    .await
                    .context("Failed to parse NRF update response")?;
                Ok(Some(profile))
            }
            StatusCode::NO_CONTENT => Ok(None),
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF update failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn discover_nf(
        &self,
        target_nf_type: NfType,
        query_params: Option<HashMap<String, String>>,
    ) -> Result<SearchResult> {
        let mut url = format!(
            "{}/nnrf-disc/v1/nf-instances?target-nf-type={}",
            self.base_url, target_nf_type
        );

        if let Some(params) = query_params {
            for (key, value) in params {
                url.push_str(&format!("&{}={}", key, value));
            }
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send discovery request to NRF")?;

        match response.status() {
            StatusCode::OK => {
                let search_result: SearchResult = response
                    .json()
                    .await
                    .context("Failed to parse NRF discovery response")?;

                tracing::info!(
                    "Discovered {} instances of {} from NRF",
                    search_result.nf_instances.len(),
                    target_nf_type
                );

                Ok(search_result)
            }
            StatusCode::NOT_FOUND => Ok(SearchResult {
                validity_period: None,
                nf_instances: vec![],
                search_id: None,
                num_nf_inst_complete: Some(0),
            }),
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF discovery failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn deregister_nf(&self, nf_instance_id: &str) -> Result<()> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.base_url, nf_instance_id
        );

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send deregistration request to NRF")?;

        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::OK => {
                tracing::info!(
                    "Successfully deregistered NF instance {} from NRF",
                    nf_instance_id
                );
                Ok(())
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF deregistration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
