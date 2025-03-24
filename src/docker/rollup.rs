use blueprint_sdk as sdk;

use crate::RollupConfig;
use crate::deployer::config::ConfigGenerator;
use crate::deployer::rollup::{DeploymentConfig, RollupDeployer};
use crate::docker::espresso::EspressoDockerManager;
use anyhow::{Result, anyhow};
use sdk::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Status of a rollup
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RollupStatus {
    /// Rollup is being created
    Creating,
    /// Rollup is created but not running
    Created,
    /// Rollup is being started
    Starting,
    /// Rollup is running
    Running,
    /// Rollup is being stopped
    Stopping,
    /// Rollup is stopped
    Stopped,
    /// Rollup is being deleted
    Deleting,
    /// Rollup creation failed
    Failed(String),
}

impl std::fmt::Display for RollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RollupStatus::Creating => write!(f, "Creating"),
            RollupStatus::Created => write!(f, "Created"),
            RollupStatus::Starting => write!(f, "Starting"),
            RollupStatus::Running => write!(f, "Running"),
            RollupStatus::Stopping => write!(f, "Stopping"),
            RollupStatus::Stopped => write!(f, "Stopped"),
            RollupStatus::Deleting => write!(f, "Deleting"),
            RollupStatus::Failed(reason) => write!(f, "Failed: {}", reason),
        }
    }
}

/// Rollup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupInfo {
    /// Service ID
    pub service_id: u64,
    /// Rollup ID
    pub rollup_id: String,
    /// VM ID
    pub vm_id: String,
    /// Rollup configuration
    pub config: RollupConfig,
    /// Rollup status
    pub status: RollupStatus,
    /// Creation timestamp
    pub created_at: String,
    /// Workspace directory
    pub workspace_dir: PathBuf,
    /// Config directory
    pub config_dir: PathBuf,
}

/// Rollup manager for managing rollups
pub struct RollupManager {
    /// Map of rollup ID to rollup information
    rollups: Arc<RwLock<HashMap<String, RollupInfo>>>,
}

impl RollupManager {
    /// Create a new rollup manager
    pub fn new() -> Self {
        Self {
            rollups: Arc::new(RwLock::const_new(HashMap::new())),
        }
    }

    /// Create a new rollup
    pub async fn create_rollup(
        &self,
        service_id: u64,
        rollup_id: &str,
        vm_id: &str,
        config: RollupConfig,
        workspace_dir: PathBuf,
        config_dir: PathBuf,
    ) -> Result<String> {
        // Update status to Creating
        let info = RollupInfo {
            service_id,
            rollup_id: rollup_id.to_string(),
            vm_id: vm_id.to_string(),
            config: config.clone(),
            status: RollupStatus::Creating,
            created_at: chrono::Utc::now().to_rfc3339(),
            workspace_dir: workspace_dir.clone(),
            config_dir: config_dir.clone(),
        };

        // Store the rollup information
        self.rollups
            .write()
            .await
            .insert(rollup_id.to_string(), info);

        info!("Deploying contracts for rollup {}", rollup_id);

        // Get deployment keys from environment variables
        let private_key = std::env::var("DEPLOYER_PRIVATE_KEY")
            .map_err(|_| anyhow!("DEPLOYER_PRIVATE_KEY environment variable not set"))?;
        let arbiscan_api_key = std::env::var("ARBISCAN_API_KEY")
            .map_err(|_| anyhow!("ARBISCAN_API_KEY environment variable not set"))?;
        let arbitrum_rpc_url = std::env::var("ARBITRUM_RPC_URL")
            .map_err(|_| anyhow!("ARBITRUM_RPC_URL environment variable not set"))?;

        // Create deployment config
        let deployment_config = DeploymentConfig::new(
            &config,
            &private_key,
            &arbiscan_api_key,
            workspace_dir.clone(),
        );

        // Create deployer and deploy contracts
        let deployer = RollupDeployer::new(deployment_config);

        // Try to deploy contracts
        let deployment_result = match deployer.deploy().await {
            Ok(result) => {
                info!(
                    "Contracts deployed successfully for rollup {}. Rollup proxy: {}",
                    rollup_id, result.rollup_proxy_address
                );
                result
            }
            Err(e) => {
                error!("Failed to deploy contracts: {}", e);

                // Update status to Failed
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status =
                        RollupStatus::Failed(format!("Contract deployment failed: {}", e));
                }

                return Err(anyhow!("Failed to deploy contracts: {}", e));
            }
        };

        // Get validator and batch poster keys from environment
        let validator_key = std::env::var("VALIDATOR_PRIVATE_KEY")
            .map_err(|_| anyhow!("VALIDATOR_PRIVATE_KEY environment variable not set"))?;
        let batch_poster_key = std::env::var("BATCH_POSTER_PRIVATE_KEY")
            .map_err(|_| anyhow!("BATCH_POSTER_PRIVATE_KEY environment variable not set"))?;

        // Use ConfigGenerator to generate all configuration files
        let config_generator = ConfigGenerator::new(
            &config_dir,
            &workspace_dir,
            config.chain_id,
            deployment_result.rollup_proxy_address,
            deployment_result.upgrade_executor_address,
            deployment_result.deployment_block,
            validator_key,
            batch_poster_key,
            arbitrum_rpc_url,
        );

        // Generate all configurations including docker-compose.yml
        match config_generator.generate_configs() {
            Ok(_) => {
                info!(
                    "Generated configuration files successfully for rollup {}",
                    rollup_id
                );
            }
            Err(e) => {
                error!("Failed to generate configuration files: {}", e);

                // Update status to Failed
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status = RollupStatus::Failed(format!("Config generation failed: {}", e));
                }

                return Err(anyhow!("Failed to generate config files: {}", e));
            }
        }

        // Update status to Created
        let mut registry = self.rollups.write().await;
        if let Some(info) = registry.get_mut(rollup_id) {
            info.status = RollupStatus::Created;
        }

        info!("Rollup {} created successfully.", rollup_id);

        Ok(rollup_id.to_string())
    }

    /// Start a rollup
    pub async fn start_rollup(&self, rollup_id: &str) -> Result<()> {
        // Get rollup information
        let registry = self.rollups.read().await;
        let info = registry
            .get(rollup_id)
            .ok_or_else(|| anyhow!("Rollup not found"))?
            .clone();
        drop(registry);

        // Update status to Starting
        {
            let mut registry = self.rollups.write().await;
            if let Some(info) = registry.get_mut(rollup_id) {
                info.status = RollupStatus::Starting;
            }
        }

        // Create and start the Docker manager based on rollup type
        let mut manager = EspressoDockerManager::new(
            info.workspace_dir.clone(),
            info.config_dir.clone(),
            &info.vm_id,
        );

        // Start the manager
        match manager.start().await {
            Ok(_) => {
                // Update the status
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status = RollupStatus::Running;
                }
                Ok(())
            }
            Err(e) => {
                // Update the status
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status = RollupStatus::Failed(e.to_string());
                }
                Err(e)
            }
        }
    }

    /// Stop a rollup
    pub async fn stop_rollup(&self, rollup_id: &str) -> Result<()> {
        // Get rollup information
        let registry = self.rollups.read().await;
        let info = registry
            .get(rollup_id)
            .ok_or_else(|| anyhow!("Rollup not found"))?
            .clone();
        drop(registry);

        // Update status to Stopping
        {
            let mut registry = self.rollups.write().await;
            if let Some(info) = registry.get_mut(rollup_id) {
                info.status = RollupStatus::Deleting;
            }
        }

        // Create and stop the Docker manager based on rollup type
        let manager = EspressoDockerManager::new(
            info.workspace_dir.clone(),
            info.config_dir.clone(),
            &info.vm_id,
        );

        // Stop the manager
        match manager.stop().await {
            Ok(_) => {
                // Update the status
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status = RollupStatus::Stopped;
                }
                Ok(())
            }
            Err(e) => {
                // Update the status
                let mut registry = self.rollups.write().await;
                if let Some(info) = registry.get_mut(rollup_id) {
                    info.status = RollupStatus::Failed(e.to_string());
                }
                Err(e)
            }
        }
    }

    /// Delete a rollup
    pub async fn delete_rollup(&self, rollup_id: &str) -> Result<()> {
        // First stop the rollup if it's running
        let registry = self.rollups.read().await;
        let info = registry
            .get(rollup_id)
            .ok_or_else(|| anyhow!("Rollup not found"))?;

        if info.status == RollupStatus::Running {
            drop(registry);
            self.stop_rollup(rollup_id).await?;
        } else {
            drop(registry);
        }

        // Remove the rollup from the registry
        self.rollups.write().await.remove(rollup_id);

        Ok(())
    }

    /// Get a rollup by ID
    pub async fn get_rollup(&self, rollup_id: &str) -> Option<RollupInfo> {
        self.rollups.read().await.get(rollup_id).cloned()
    }

    /// Get a rollup by VM ID
    pub async fn get_rollup_by_vm_id(&self, vm_id: &str) -> Option<RollupInfo> {
        self.rollups
            .read()
            .await
            .values()
            .find(|info| info.vm_id == vm_id)
            .cloned()
    }

    /// Get a rollup by service ID
    pub async fn get_rollup_by_service_id(&self, service_id: u64) -> Option<RollupInfo> {
        self.rollups
            .read()
            .await
            .values()
            .find(|info| info.service_id == service_id)
            .cloned()
    }

    /// List all rollups
    pub async fn list_rollups(&self) -> Vec<RollupInfo> {
        self.rollups.read().await.values().cloned().collect()
    }

    /// Get the status of a rollup
    pub async fn get_rollup_status(&self, rollup_id: &str) -> Result<RollupStatus> {
        // Get rollup information
        let registry = self.rollups.read().await;
        let info = registry
            .get(rollup_id)
            .ok_or_else(|| anyhow!("Rollup not found"))?;

        Ok(info.status.clone())
    }

    /// Update the status of a rollup
    pub async fn update_rollup_status(&self, rollup_id: &str, status: RollupStatus) -> Result<()> {
        // Get rollup information
        let mut registry = self.rollups.write().await;
        let info = registry
            .get_mut(rollup_id)
            .ok_or_else(|| anyhow!("Rollup not found"))?;

        // Update the status
        info.status = status;

        Ok(())
    }
}

impl Default for RollupManager {
    fn default() -> Self {
        Self::new()
    }
}
