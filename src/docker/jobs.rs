use blueprint_sdk as sdk;

use crate::{RollupConfig, RollupConfigParams};
use anyhow::{Result, anyhow};
use sdk::tangle::extract::{List, ServiceId, TangleArg, TangleResult};
use uuid::Uuid;

/// Create a new Docker-based rollup
pub async fn create_docker_rollup(
    ServiceId(service_id): ServiceId,
    TangleArg(List(config_bytes)): TangleArg<List<u8>>,
) -> Result<TangleResult<bool>> {
    // Deserialize the config bytes
    let config_params: RollupConfigParams = serde_json::from_slice(&config_bytes)
        .map_err(|e| anyhow!("Failed to deserialize rollup config: {}", e))?;

    // Convert to RollupConfig
    let config = RollupConfig::from(config_params);

    sdk::info!(
        "Creating Docker-based rollup for service_id: {}",
        service_id
    );
    sdk::info!("Docker rollup config: {:?}", config);

    // Create a unique VM ID
    let rollup_id = Uuid::new_v4().to_string();
    let vm_id = format!("docker-rollup-{}-{}", service_id, rollup_id);

    // Create and start the Docker-based rollup
    let created = crate::create_rollup(service_id, rollup_id, vm_id, config).await?;

    sdk::info!("Docker rollup created: {:?}", created);
    Ok(TangleResult(created))
}

/// Start an existing Docker-based rollup
pub async fn start_docker_rollup(
    ServiceId(service_id): ServiceId,
    _: TangleArg<()>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Starting Docker-based rollup for service_id: {}",
        service_id
    );

    // Start the Docker-based rollup
    let started = crate::start_rollup(service_id).await?;

    sdk::info!("Docker rollup started: {:?}", started);
    Ok(TangleResult(started))
}

/// Stop an existing Docker-based rollup
pub async fn stop_docker_rollup(
    ServiceId(service_id): ServiceId,
    _: TangleArg<()>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Stopping Docker-based rollup for service_id: {}",
        service_id
    );

    // Stop the Docker-based rollup
    let stopped = crate::stop_rollup(service_id).await?;

    sdk::info!("Docker rollup stopped: {:?}", stopped);
    Ok(TangleResult(stopped))
}

/// Delete a Docker-based rollup
pub async fn delete_docker_rollup(
    ServiceId(service_id): ServiceId,
    _: TangleArg<()>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Deleting Docker-based rollup for service_id: {}",
        service_id
    );

    // Delete the Docker-based rollup
    let deleted = crate::delete_rollup(service_id).await?;
    Ok(TangleResult(deleted))
}
