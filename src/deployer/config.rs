use blueprint_sdk as sdk;

use anyhow::Result;
use sdk::info;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration generator for Espresso node
pub struct ConfigGenerator {
    config_dir: PathBuf,
    workspace_dir: PathBuf,
    chain_id: u64,
    rollup_address: String,
    upgrade_executor_address: String,
    deployment_block: u64,
    validator_key: String,
    batch_poster_key: String,
    arbitrum_rpc_url: String,
}

impl ConfigGenerator {
    /// Create a new config generator
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        config_dir: P,
        workspace_dir: Q,
        chain_id: u64,
        rollup_address: String,
        upgrade_executor_address: String,
        deployment_block: u64,
        validator_key: String,
        batch_poster_key: String,
        arbitrum_rpc_url: String,
    ) -> Self {
        Self {
            config_dir: config_dir.as_ref().to_path_buf(),
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
            chain_id,
            rollup_address,
            upgrade_executor_address,
            deployment_block,
            validator_key,
            batch_poster_key,
            arbitrum_rpc_url,
        }
    }

    /// Generate all configuration files
    pub fn generate_configs(&self) -> Result<()> {
        // Create directories if they don't exist
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.workspace_dir)?;

        // Copy the template config files and substitute values
        self.copy_and_update_l2_chain_info()?;
        self.copy_and_update_full_node()?;
        self.copy_and_update_validation_node_config()?;
        self.copy_jwt_file()?;
        self.copy_docker_compose()?;

        info!(
            "Configuration files generated in {}",
            self.config_dir.display()
        );
        Ok(())
    }

    fn copy_and_update_l2_chain_info(&self) -> Result<()> {
        // Read the template file
        let template = include_str!("config/l2_chain_info.json");

        // Replace placeholders with actual values
        let content = template
            .replace("10000000", &self.chain_id.to_string())
            .replace(
                "INITIAL_CHAIN_OWNER_ADDRESS",
                &std::env::var("INITIAL_CHAIN_OWNER").unwrap_or_default(),
            )
            .replace(
                "BRIDGE_ADDRESS",
                &std::env::var("BRIDGE_ADDRESS").unwrap_or_default(),
            )
            .replace(
                "INBOX_ADDRESS",
                &std::env::var("INBOX_ADDRESS").unwrap_or_default(),
            )
            .replace(
                "SEQUENCER_INBOX_ADDRESS",
                &std::env::var("SEQUENCER_INBOX_ADDRESS").unwrap_or_default(),
            )
            .replace("100000000", &self.deployment_block.to_string())
            .replace("ROLLUP_ADDRESS", &self.rollup_address)
            .replace("UPGRADE_EXECUTOR_ADDRESS", &self.upgrade_executor_address)
            .replace(
                "VALIDATOR_UTILS_ADDRESS",
                &std::env::var("VALIDATOR_UTILS_ADDRESS").unwrap_or_default(),
            )
            .replace(
                "VALIDATOR_WALLET_CREATOR_ADDRESS",
                &std::env::var("VALIDATOR_WALLET_CREATOR_ADDRESS").unwrap_or_default(),
            );

        // Write to the output file
        let output_path = self.config_dir.join("l2_chain_info.json");
        fs::write(&output_path, content)?;

        info!("Generated l2_chain_info.json at {}", output_path.display());
        Ok(())
    }

    fn copy_and_update_full_node(&self) -> Result<()> {
        // Read the template file
        let template = include_str!("config/full_node.json");

        // Write to the output file
        let output_path = self.config_dir.join("full_node.json");
        fs::write(&output_path, template)?;

        info!("Generated full_node.json at {}", output_path.display());
        Ok(())
    }

    fn copy_and_update_validation_node_config(&self) -> Result<()> {
        // Read the template file
        let template = include_str!("config/validation_node_config.json");

        // Write to the output file
        let output_path = self.config_dir.join("validation_node_config.json");
        fs::write(&output_path, template)?;

        info!(
            "Generated validation_node_config.json at {}",
            output_path.display()
        );
        Ok(())
    }

    fn copy_jwt_file(&self) -> Result<()> {
        // Read the template file
        let template = include_str!("config/val_jwt.hex");

        // Write to the output file
        let output_path = self.config_dir.join("val_jwt.hex");
        fs::write(&output_path, template)?;

        info!("Generated JWT file at {}", output_path.display());
        Ok(())
    }

    fn copy_docker_compose(&self) -> Result<()> {
        // Read the docker-compose template
        let template = include_str!("config/docker-compose.yml");

        // Write to the workspace directory
        let output_path = self.workspace_dir.join("docker-compose.yml");
        fs::write(&output_path, template)?;

        info!("Generated docker-compose.yml at {}", output_path.display());
        Ok(())
    }
}
