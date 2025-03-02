use anyhow::{anyhow, Result};
use dockworker::parser::ComposeParser;
use dockworker::{ComposeConfig, DockerBuilder, Service};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{error, info};

/// Options for Docker container execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeOptions {
    /// Path to the docker-compose.yml file
    pub compose_file_path: PathBuf,

    /// Path to the config directory
    pub config_dir: PathBuf,

    /// Project name for docker-compose
    pub project_name: String,
}

/// Manager for Docker containers using docker-compose
pub struct DockerComposeManager {
    docker: DockerBuilder,
    options: DockerComposeOptions,
    container_ids: HashMap<String, String>,
}

impl DockerComposeManager {
    /// Create a new Docker compose manager
    pub async fn new(options: DockerComposeOptions) -> Result<Self> {
        let docker = DockerBuilder::new().await?;
        Ok(Self {
            docker,
            options,
            container_ids: HashMap::new(),
        })
    }

    /// Start the containers defined in the docker-compose.yml file
    pub async fn start_containers(&mut self) -> Result<()> {
        info!(
            "Starting containers from compose file: {}",
            self.options.compose_file_path.display()
        );

        // Check if compose file exists
        if !self.options.compose_file_path.exists() {
            return Err(anyhow!(
                "Docker compose file not found at {}",
                self.options.compose_file_path.display()
            ));
        }

        // Parse the compose file
        let compose_content = std::fs::read_to_string(&self.options.compose_file_path)?;
        let config = ComposeParser::new()
            .parse(&mut compose_content.as_bytes())
            .map_err(|e| anyhow!("Failed to parse compose file: {}", e))?;

        // Create a network for services
        let network_name = format!("network-{}", self.options.project_name);

        // Prepare labels for tracking
        let mut labels = HashMap::new();
        labels.insert("project".to_string(), self.options.project_name.clone());

        // Create the network with retry mechanism
        self.docker
            .create_network_with_retry(
                &network_name,
                3,
                Duration::from_secs(2),
                Some(labels.clone()),
            )
            .await
            .map_err(|e| anyhow!("Failed to create network: {}", e))?;

        // Prepare the compose configuration
        let mut services = HashMap::new();

        // Convert the parsed services to the format expected by dockworker
        for (service_name, parsed_service) in &config.services {
            // Create service with required configuration
            let service = Service {
                image: parsed_service.image.clone(),
                ports: parsed_service.ports.clone(),
                environment: parsed_service.environment.clone(),
                networks: Some(vec![network_name.clone()]),
                volumes: parsed_service.volumes.clone(),
                requirements: parsed_service.requirements.clone(),
                depends_on: parsed_service.depends_on.clone(),
                healthcheck: parsed_service.healthcheck.clone(),
                restart: parsed_service.restart.clone(),
                command: parsed_service.command.clone(),
                user: parsed_service.user.clone(),
                labels: Some(labels.clone()),
                platform: parsed_service.platform.clone(),
                env_file: parsed_service.env_file.clone(),
                build: parsed_service.build.clone(),
            };

            services.insert(service_name.clone(), service);
        }

        // Create the compose configuration
        let mut compose_config = ComposeConfig {
            version: "3".to_string(),
            services,
            volumes: config.volumes.clone(),
        };

        // Deploy the compose configuration
        let container_ids = self
            .docker
            .deploy_compose(&mut compose_config)
            .await
            .map_err(|e| anyhow!("Failed to deploy compose configuration: {}", e))?;

        // Store container IDs
        for (name, id) in container_ids {
            self.container_ids.insert(name, id);
        }

        info!("All containers started successfully");
        Ok(())
    }

    /// Stop the containers defined in the docker-compose.yml file
    pub async fn stop_containers(&self) -> Result<()> {
        info!(
            "Stopping containers from compose file: {}",
            self.options.compose_file_path.display()
        );

        // First try using the dockworker API
        let mut api_success = true;
        let mut api_error = String::new();

        // Stop each container using the API
        for (service_name, container_id) in &self.container_ids {
            match self
                .docker
                .get_client()
                .stop_container(container_id, None)
                .await
            {
                Ok(_) => {
                    info!("Stopped container for service: {}", service_name);

                    // Remove the container
                    if let Err(e) = self
                        .docker
                        .get_client()
                        .remove_container(container_id, None)
                        .await
                    {
                        api_success = false;
                        api_error = format!("Failed to remove container {}: {}", service_name, e);
                        error!("{}", api_error);
                        break;
                    }
                }
                Err(e) => {
                    api_success = false;
                    api_error = format!("Failed to stop container {}: {}", service_name, e);
                    error!("{}", api_error);
                    break;
                }
            }
        }

        // Try to remove the network
        if api_success {
            let network_name = format!("network-{}", self.options.project_name);
            if let Err(e) = self.docker.get_client().remove_network(&network_name).await {
                api_success = false;
                api_error = format!("Failed to remove network: {}", e);
                error!("{}", api_error);
            }
        }

        // If the API approach failed, fall back to docker-compose down command
        if !api_success {
            info!(
                "Dockworker API failed: {}. Falling back to docker-compose command",
                api_error
            );

            let output = std::process::Command::new("docker-compose")
                .arg("-f")
                .arg(&self.options.compose_file_path)
                .arg("-p")
                .arg(&self.options.project_name)
                .arg("down")
                .output()?;

            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!(
                    "Failed to stop containers using docker-compose: {}",
                    error_msg
                );
                return Err(anyhow!("Failed to stop containers: {}", error_msg));
            }
        }

        info!("All containers stopped successfully");
        Ok(())
    }

    /// Get the status of a specific service
    pub async fn get_service_status(&self, service_name: &str) -> Result<String> {
        if let Some(container_id) = self.container_ids.get(service_name) {
            // Use Docker API to inspect container
            let inspect = self
                .docker
                .get_client()
                .inspect_container(container_id, None)
                .await
                .map_err(|e| anyhow!("Failed to inspect container: {}", e))?;

            if let Some(state) = inspect.state {
                if let Some(status) = state.status {
                    return Ok(status.to_string());
                }
            }

            Err(anyhow!("Could not determine container status"))
        } else {
            Err(anyhow!(
                "Container ID not found for service {}",
                service_name
            ))
        }
    }

    /// Get the logs for a specific service
    pub async fn get_service_logs(&self, service_name: &str) -> Result<String> {
        if let Some(container_id) = self.container_ids.get(service_name) {
            // Get logs using the Docker API
            let logs = self
                .docker
                .get_container_logs(container_id)
                .await
                .map_err(|e| anyhow!("Failed to get container logs: {}", e))?;

            Ok(logs)
        } else {
            // Fall back to docker-compose command if container ID not found
            let output = std::process::Command::new("docker-compose")
                .arg("-f")
                .arg(&self.options.compose_file_path)
                .arg("-p")
                .arg(&self.options.project_name)
                .arg("logs")
                .arg(service_name)
                .output()?;

            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!(
                    "Failed to get logs for service {}: {}",
                    service_name, error_msg
                );
                return Err(anyhow!("Failed to get logs for service {}", service_name));
            }

            let logs = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(logs)
        }
    }

    /// Execute a command in a specific service container
    pub async fn exec_command(&self, service_name: &str, command: &[&str]) -> Result<String> {
        if let Some(container_id) = self.container_ids.get(service_name) {
            // Execute the command
            let output = self
                .docker
                .exec_in_container(container_id, command.to_vec(), None)
                .await
                .map_err(|e| anyhow!("Failed to execute command: {}", e))?;

            Ok(output)
        } else {
            Err(anyhow!(
                "Container ID not found for service {}",
                service_name
            ))
        }
    }
}
