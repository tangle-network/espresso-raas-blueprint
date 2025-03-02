use crate::{RollupConfig, RollupConfigParams, ServiceContext};
use anyhow::{anyhow, Result};
use api::services::events::JobCalled;
use blueprint_sdk::event_listeners::tangle::events::TangleEventListener;
use blueprint_sdk::event_listeners::tangle::services::{
    services_post_processor, services_pre_processor,
};
use blueprint_sdk::logging;
use blueprint_sdk::tangle_subxt::tangle_testnet_runtime::api;
use uuid::Uuid;

/// Create a new Docker-based rollup
#[blueprint_sdk::job(
    id = 3,
    params(config_bytes),
    result(success),
    event_listener(
        listener = TangleEventListener::<ServiceContext, JobCalled>,
        pre_processor = services_pre_processor,
        post_processor = services_post_processor,
    ),
)]
pub async fn create_docker_rollup(config_bytes: Vec<u8>, context: ServiceContext) -> Result<bool> {
    let service_id = context.call_id.unwrap_or(0);

    // Deserialize the config bytes
    let config_params: RollupConfigParams = serde_json::from_slice(&config_bytes)
        .map_err(|e| anyhow!("Failed to deserialize rollup config: {}", e))?;

    // Convert to RollupConfig
    let config = RollupConfig::from(config_params);

    logging::info!(
        "Creating Docker-based rollup for service_id: {}",
        service_id
    );
    logging::info!("Docker rollup config: {:?}", config);

    // Create a unique VM ID
    let rollup_id = Uuid::new_v4().to_string();
    let vm_id = format!("docker-rollup-{}-{}", service_id, rollup_id);

    // Create and start the Docker-based rollup
    super::helpers::create_rollup(service_id, rollup_id, vm_id, config).await
}

/// Start an existing Docker-based rollup
#[blueprint_sdk::job(
    id = 4,
    params(service_id),
    result(success),
    event_listener(
        listener = TangleEventListener::<ServiceContext, JobCalled>,
        pre_processor = services_pre_processor,
        post_processor = services_post_processor,
    ),
)]
pub async fn start_docker_rollup(service_id: u64, context: ServiceContext) -> Result<bool> {
    logging::info!(
        "Starting Docker-based rollup for service_id: {}",
        service_id
    );

    // Start the Docker-based rollup
    super::helpers::start_rollup(service_id).await
}

/// Stop an existing Docker-based rollup
#[blueprint_sdk::job(
    id = 5,
    params(service_id),
    result(success),
    event_listener(
        listener = TangleEventListener::<ServiceContext, JobCalled>,
        pre_processor = services_pre_processor,
        post_processor = services_post_processor,
    ),
)]
pub async fn stop_docker_rollup(service_id: u64, context: ServiceContext) -> Result<bool> {
    logging::info!(
        "Stopping Docker-based rollup for service_id: {}",
        service_id
    );

    // Stop the Docker-based rollup
    super::helpers::stop_rollup(service_id).await
}

/// Delete a Docker-based rollup
#[blueprint_sdk::job(
    id = 6,
    params(service_id),
    result(success),
    event_listener(
        listener = TangleEventListener::<ServiceContext, JobCalled>,
        pre_processor = services_pre_processor,
        post_processor = services_post_processor,
    ),
)]
pub async fn delete_docker_rollup(service_id: u64, context: ServiceContext) -> Result<bool> {
    logging::info!(
        "Deleting Docker-based rollup for service_id: {}",
        service_id
    );

    // Delete the Docker-based rollup
    super::helpers::delete_rollup(service_id).await
}
