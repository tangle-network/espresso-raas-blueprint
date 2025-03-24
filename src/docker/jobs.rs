use blueprint_sdk as sdk;

use crate::{RollupConfig, RollupConfigParams};
use anyhow::Result;
use sdk::tangle::extract::{ServiceId, TangleArg, TangleResult};
use uuid::Uuid;

/// Create a new Docker-based rollup
///
/// Returns the ID of the created rollup
pub async fn create_docker_rollup(
    ServiceId(service_id): ServiceId,
    TangleArg(config_params): TangleArg<RollupConfigParams>,
) -> Result<TangleResult<String>> {
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
    let created = crate::create_rollup(service_id, &rollup_id, &vm_id, config).await?;

    sdk::info!("Docker rollup created: {:?}", created);
    Ok(TangleResult(rollup_id))
}

/// Start an existing Docker-based rollup
pub async fn start_docker_rollup(
    ServiceId(service_id): ServiceId,
    TangleArg(rollup_id): TangleArg<String>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Starting Docker-based rollup for service_id: {} with rollup_id: {}",
        service_id,
        rollup_id
    );

    // Start the Docker-based rollup
    let started = crate::start_rollup(&rollup_id).await?;

    sdk::info!("Docker rollup started: {:?}", started);
    Ok(TangleResult(started))
}

/// Stop an existing Docker-based rollup
pub async fn stop_docker_rollup(
    ServiceId(service_id): ServiceId,
    TangleArg(rollup_id): TangleArg<String>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Stopping Docker-based rollup for service_id: {} with rollup_id: {}",
        service_id,
        rollup_id
    );

    // Stop the Docker-based rollup
    let stopped = crate::stop_rollup(&rollup_id).await?;

    sdk::info!("Docker rollup stopped: {:?}", stopped);
    Ok(TangleResult(stopped))
}

/// Delete a Docker-based rollup
pub async fn delete_docker_rollup(
    ServiceId(service_id): ServiceId,
    TangleArg(rollup_id): TangleArg<String>,
) -> Result<TangleResult<bool>> {
    sdk::info!(
        "Deleting Docker-based rollup for service_id: {} with rollup_id: {}",
        service_id,
        rollup_id
    );

    // Delete the Docker-based rollup
    let deleted = crate::delete_rollup(&rollup_id).await?;
    Ok(TangleResult(deleted))
}
