use blueprint_sdk as sdk;

use crate::docker::container::{DockerComposeManager, DockerComposeOptions};
use anyhow::{Result, anyhow};
use sdk::info;
use std::path::{Path, PathBuf};

/// Connect Docker with the Espresso configuration generator
pub struct EspressoDockerManager {
    compose_manager: Option<DockerComposeManager>,
    workspace_dir: PathBuf,
    config_dir: PathBuf,
    vm_id: String,
}

impl EspressoDockerManager {
    /// Create a new Espresso Docker manager
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        workspace_dir: P,
        config_dir: Q,
        vm_id: &str,
    ) -> Self {
        Self {
            compose_manager: None,
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
            config_dir: config_dir.as_ref().to_path_buf(),
            vm_id: vm_id.to_string(),
        }
    }

    /// Initialize and start the Docker containers
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting Espresso Docker containers for VM ID: {}",
            self.vm_id
        );

        let compose_file_path = self.workspace_dir.join("docker-compose.yml");

        // Create Docker compose options
        let options = DockerComposeOptions {
            compose_file_path,
            config_dir: self.config_dir.clone(),
            project_name: format!("espresso-{}", self.vm_id),
        };

        // Create and initialize the Docker compose manager
        let mut compose_manager = DockerComposeManager::new(options).await?;

        // Start the containers
        compose_manager.start_containers().await?;

        // Store the compose manager
        self.compose_manager = Some(compose_manager);

        info!("Espresso Docker containers started successfully");
        Ok(())
    }

    /// Stop the Docker containers
    pub async fn stop(&self) -> Result<()> {
        info!(
            "Stopping Espresso Docker containers for VM ID: {}",
            self.vm_id
        );

        if let Some(compose_manager) = &self.compose_manager {
            compose_manager.stop_containers().await?;
            info!("Espresso Docker containers stopped successfully");
            Ok(())
        } else {
            Err(anyhow!("Docker compose manager not initialized"))
        }
    }

    /// Get the status of the Espresso node
    pub async fn get_status(&self) -> Result<String> {
        if let Some(compose_manager) = &self.compose_manager {
            compose_manager.get_service_status("nitro").await
        } else {
            Ok("NotRunning".to_string())
        }
    }

    /// Get the logs of the Espresso node
    pub async fn get_logs(&self) -> Result<String> {
        if let Some(compose_manager) = &self.compose_manager {
            compose_manager.get_service_logs("nitro").await
        } else {
            Err(anyhow!("Docker compose manager not initialized"))
        }
    }

    /// Execute a command in the Espresso node container
    pub async fn exec_command(&self, command: &[&str]) -> Result<String> {
        if let Some(compose_manager) = &self.compose_manager {
            compose_manager.exec_command("nitro", command).await
        } else {
            Err(anyhow!("Docker compose manager not initialized"))
        }
    }
}
