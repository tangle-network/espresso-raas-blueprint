use blueprint_sdk as sdk;

use crate::{RollupConfig, docker::rollup::RollupManager};
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use sdk::{error, info};
use std::collections::HashMap;
use std::path::PathBuf;

// Singleton for managing rollups
lazy_static! {
    pub static ref ROLLUP_MANAGER: RollupManager = RollupManager::new();
}

/// Create a new rollup
pub async fn create_rollup(
    service_id: u64,
    rollup_id: &str,
    vm_id: &str,
    config: RollupConfig,
) -> Result<bool> {
    info!(
        "Creating rollup for service_id: {}, vm_id: {}",
        service_id, vm_id
    );

    // Set workspace and config directories based on VM ID
    let workspace_dir = PathBuf::from(format!("/tmp/espresso/{}/workspace", vm_id));
    let config_dir = PathBuf::from(format!("/tmp/espresso/{}/config", vm_id));

    // Create directories if they don't exist
    std::fs::create_dir_all(&workspace_dir).map_err(|e| {
        anyhow!(
            "Failed to create workspace directory {}: {}",
            workspace_dir.display(),
            e
        )
    })?;
    std::fs::create_dir_all(&config_dir).map_err(|e| {
        anyhow!(
            "Failed to create config directory {}: {}",
            config_dir.display(),
            e
        )
    })?;

    // Create rollup in the manager
    match ROLLUP_MANAGER
        .create_rollup(
            service_id,
            rollup_id,
            vm_id,
            config,
            workspace_dir,
            config_dir,
        )
        .await
    {
        Ok(rollup_id) => {
            info!("Created rollup with rollup_id: {}", rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to create rollup: {}", e);
            Err(anyhow!("Failed to create rollup: {}", e))
        }
    }
}

/// Start a rollup
pub async fn start_rollup(rollup_id: &str) -> Result<bool> {
    info!("Starting rollup for rollup_id: {}", rollup_id);

    // Get rollup by service ID
    let rollup = ROLLUP_MANAGER
        .get_rollup(rollup_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for rollup_id: {}", rollup_id))?;

    // Start the rollup
    match ROLLUP_MANAGER.start_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Started rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to start rollup: {}", e);
            Err(anyhow!("Failed to start rollup: {}", e))
        }
    }
}

/// Start a rollup by service ID
pub async fn start_rollup_by_service_id(service_id: u64) -> Result<bool> {
    info!("Starting rollup for service_id: {}", service_id);

    // Get rollup by service ID
    let rollup = ROLLUP_MANAGER
        .get_rollup_by_service_id(service_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for service_id: {}", service_id))?;

    // Start the rollup
    match ROLLUP_MANAGER.start_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Started rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to start rollup: {}", e);
            Err(anyhow!("Failed to start rollup: {}", e))
        }
    }
}

/// Stop a rollup by rollup ID
pub async fn stop_rollup(rollup_id: &str) -> Result<bool> {
    info!("Stopping rollup for rollup_id: {}", rollup_id);

    // Get rollup by rollup ID
    let rollup = ROLLUP_MANAGER
        .get_rollup(rollup_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for rollup_id: {}", rollup_id))?;

    // Stop the rollup
    match ROLLUP_MANAGER.stop_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Stopped rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to stop rollup: {}", e);
            Err(anyhow!("Failed to stop rollup: {}", e))
        }
    }
}

/// Stop a rollup by service ID
pub async fn stop_rollup_by_service_id(service_id: u64) -> Result<bool> {
    info!("Stopping rollup for service_id: {}", service_id);

    // Get rollup by service ID
    let rollup = ROLLUP_MANAGER
        .get_rollup_by_service_id(service_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for service_id: {}", service_id))?;

    // Stop the rollup
    match ROLLUP_MANAGER.stop_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Stopped rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to stop rollup: {}", e);
            Err(anyhow!("Failed to stop rollup: {}", e))
        }
    }
}

/// Delete a rollup by rollup ID
pub async fn delete_rollup(rollup_id: &str) -> Result<bool> {
    info!("Deleting rollup for rollup_id: {}", rollup_id);

    // Get rollup by rollup ID
    let rollup = ROLLUP_MANAGER
        .get_rollup(rollup_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for rollup_id: {}", rollup_id))?;

    // Delete the rollup
    match ROLLUP_MANAGER.delete_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Deleted rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to delete rollup: {}", e);
            Err(anyhow!("Failed to delete rollup: {}", e))
        }
    }
}

/// Delete a rollup by service ID
pub async fn delete_rollup_by_service_id(service_id: u64) -> Result<bool> {
    info!("Deleting rollup for service_id: {}", service_id);

    // Get rollup by service ID
    let rollup = ROLLUP_MANAGER
        .get_rollup_by_service_id(service_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for service_id: {}", service_id))?;

    // Delete the rollup
    match ROLLUP_MANAGER.delete_rollup(&rollup.rollup_id).await {
        Ok(_) => {
            info!("Deleted rollup with rollup_id: {}", rollup.rollup_id);
            Ok(true)
        }
        Err(e) => {
            error!("Failed to delete rollup: {}", e);
            Err(anyhow!("Failed to delete rollup: {}", e))
        }
    }
}

/// Get the status of a rollup
pub async fn get_rollup_status(vm_id: &str) -> Result<String> {
    info!("Getting status for rollup with vm_id: {}", vm_id);

    // Get rollup by VM ID
    let rollup = ROLLUP_MANAGER
        .get_rollup_by_vm_id(vm_id)
        .await
        .ok_or_else(|| anyhow!("Rollup not found for vm_id: {}", vm_id))?;

    // Get the status
    Ok(rollup.status.to_string())
}

/// List all rollups
pub async fn list_rollups() -> Vec<HashMap<String, String>> {
    info!("Listing all rollups");

    // Get all rollups
    let rollups = ROLLUP_MANAGER.list_rollups().await;

    // Convert to a simpler format
    rollups
        .into_iter()
        .map(|rollup| {
            let mut map = HashMap::new();
            map.insert("service_id".to_string(), rollup.service_id.to_string());
            map.insert("rollup_id".to_string(), rollup.rollup_id);
            map.insert("vm_id".to_string(), rollup.vm_id);
            map.insert("status".to_string(), rollup.status.to_string());
            map.insert("created_at".to_string(), rollup.created_at);
            map
        })
        .collect()
}
