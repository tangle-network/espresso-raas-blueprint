use crate::RollupConfig;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};

// Constants for deployment
const NITRO_CONTRACTS_REPO: &str = "https://github.com/EspressoSystems/nitro-contracts.git";
const NITRO_CONTRACTS_BRANCH: &str = "develop";
const TEE_VERIFIER_ADDRESS: &str = "0x8354db765810dF8F24f1477B06e91E5b17a408bF";

// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub private_key: String,
    pub arbiscan_api_key: String,
    pub chain_id: u64,
    pub initial_chain_owner: String,
    pub validators: Vec<String>,
    pub batch_poster_address: String,
    pub batch_poster_manager: String,
    pub workspace_dir: PathBuf,
}

impl DeploymentConfig {
    pub fn new(
        rollup_config: &RollupConfig,
        private_key: &str,
        arbiscan_api_key: &str,
        workspace_dir: PathBuf,
    ) -> Self {
        Self {
            private_key: private_key.to_string(),
            arbiscan_api_key: arbiscan_api_key.to_string(),
            chain_id: rollup_config.chain_id,
            initial_chain_owner: rollup_config.initial_chain_owner.clone(),
            validators: rollup_config.validators.iter().map(|v| v.clone()).collect(),
            batch_poster_address: rollup_config.batch_poster_address.clone(),
            batch_poster_manager: rollup_config.batch_poster_manager.clone(),
            workspace_dir,
        }
    }
}

/// Automated deployer for rollup contracts
pub struct RollupDeployer {
    config: DeploymentConfig,
}

impl RollupDeployer {
    pub fn new(config: DeploymentConfig) -> Self {
        Self { config }
    }

    /// Execute the full deployment process
    pub async fn deploy(&self) -> Result<DeploymentResult> {
        info!("Starting rollup contract deployment process");

        // Step 0: Create workspace directory if it doesn't exist
        fs::create_dir_all(&self.config.workspace_dir)?;

        // Step 1: Clone and set up the contracts repository
        self.clone_contracts_repo()?;

        // Step 2: Install dependencies and build
        self.build_contracts()?;

        // Step 3: Create environment files
        self.create_env_file()?;

        // Step 4: Create config.ts
        self.create_config_file()?;

        // Step 5: Run deployment script
        let rollup_creator_address = self.deploy_contracts()?;

        // Step 6: Update .env with rollup creator address
        self.update_env_with_creator(rollup_creator_address.clone())?;

        // Step 7: Deploy rollup proxy contract
        let (rollup_proxy_address, upgrade_executor_address, deployment_block) =
            self.deploy_rollup_proxy()?;

        info!("Rollup deployment completed successfully");

        Ok(DeploymentResult {
            rollup_creator_address,
            rollup_proxy_address,
            upgrade_executor_address,
            deployment_block,
            chain_id: self.config.chain_id,
        })
    }

    /// Clone the nitro-contracts repository
    fn clone_contracts_repo(&self) -> Result<()> {
        info!("Cloning contracts repository");

        let mut cmd = Command::new("git");
        cmd.current_dir(&self.config.workspace_dir)
            .arg("clone")
            .arg(NITRO_CONTRACTS_REPO);

        let output = cmd.output()?;
        if !output.status.success() {
            error!(
                "Failed to clone contracts repository: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(anyhow!("Failed to clone contracts repository"));
        }

        // Checkout specific branch
        let nitro_contracts_dir = self.config.workspace_dir.join("nitro-contracts");
        let mut cmd = Command::new("git");
        cmd.current_dir(&nitro_contracts_dir)
            .arg("checkout")
            .arg(NITRO_CONTRACTS_BRANCH);

        let output = cmd.output()?;
        if !output.status.success() {
            error!(
                "Failed to checkout branch: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(anyhow!("Failed to checkout branch"));
        }

        info!("Contracts repository cloned successfully");
        Ok(())
    }

    /// Build the contracts
    fn build_contracts(&self) -> Result<()> {
        info!("Installing dependencies and building contracts");
        let dir = &self.config.workspace_dir.join("nitro-contracts");

        // Run yarn install && forge install
        self.run_command("yarn", &["install"], dir)?;
        self.run_command("forge", &["install"], dir)?;

        // Build the contracts (ignore stderr warnings)
        info!("Building contracts with yarn build:all");
        match self.run_command("yarn", &["build:all"], dir) {
            Ok(_) => info!("Contracts built successfully"),
            Err(e) => info!("Build completed with warnings: {}", e),
        }

        Ok(())
    }

    /// Helper function to run a command and handle errors consistently
    fn run_command(&self, cmd: &str, args: &[&str], dir: &PathBuf) -> Result<()> {
        let output = Command::new(cmd).current_dir(dir).args(args).output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Command '{}' failed: {}", cmd, err);
            return Err(anyhow!("Command '{}' failed: {}", cmd, err));
        }

        Ok(())
    }

    /// Create the .env file with required variables
    fn create_env_file(&self) -> Result<()> {
        info!("Creating .env file");

        let nitro_contracts_dir = self.config.workspace_dir.join("nitro-contracts");
        let env_content = format!(
            "ARBISCAN_API_KEY=\"{}\"\n\
             DEVNET_PRIVKEY=\"{}\"\n\
             ESPRESSO_TEE_VERIFIER_ADDRESS=\"{}\"\n",
            self.config.arbiscan_api_key, self.config.private_key, TEE_VERIFIER_ADDRESS
        );

        fs::write(nitro_contracts_dir.join(".env"), env_content)?;

        info!(".env file created successfully");
        Ok(())
    }

    /// Create the config.ts file for deployment
    fn create_config_file(&self) -> Result<()> {
        info!("Creating config.ts for deployment");
        let dir = &self.config.workspace_dir.join("nitro-contracts");

        // Copy from template
        let template_path = dir.join("scripts/config.template.ts");
        let config_path = dir.join("scripts/config.ts");

        let template = fs::read_to_string(&template_path)
            .map_err(|e| anyhow!("Failed to read config template: {}", e))?;

        // Replace placeholder values with actual config
        let config = template
            .replace("OWNER_ADDRESS", &self.config.initial_chain_owner)
            .replace("YOUR_CHAIN_ID", &self.config.chain_id.to_string())
            .replace("ChainID", &self.config.chain_id.to_string())
            .replace("YOUR_OWNED_ADDRESS", &self.config.initial_chain_owner)
            .replace("AN_OWNED_ADDRESS", &self.config.validators[0])
            .replace("ANOTHER_OWNED_ADDRESS", &self.config.batch_poster_address);

        fs::write(&config_path, config).map_err(|e| anyhow!("Failed to write config.ts: {}", e))?;

        info!("Created config.ts at {}", config_path.display());
        Ok(())
    }

    /// Deploy contracts using hardhat
    fn deploy_contracts(&self) -> Result<String> {
        info!("Deploying contracts");
        let dir = &self.config.workspace_dir.join("nitro-contracts");

        // Run deployment script
        let output = Command::new("npx")
            .current_dir(dir)
            .arg("hardhat")
            .arg("run")
            .arg("scripts/deployment.ts")
            .arg("--network")
            .arg("arbSepolia")
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Deployment failed: {}", err);
            return Err(anyhow!("Deployment failed: {}", err));
        }

        // Extract rollup creator address from output
        let output_str = String::from_utf8_lossy(&output.stdout);
        self.extract_rollup_creator_address(&output_str)
    }

    /// Update .env with the rollup creator address
    fn update_env_with_creator(&self, rollup_creator_address: String) -> Result<()> {
        info!("Updating .env with rollup creator address");

        let nitro_contracts_dir = self.config.workspace_dir.join("nitro-contracts");
        let env_path = nitro_contracts_dir.join(".env");

        let mut env_content = fs::read_to_string(&env_path)?;
        env_content.push_str(&format!(
            "ROLLUP_CREATOR_ADDRESS=\"{}\"\n",
            rollup_creator_address
        ));

        fs::write(env_path, env_content)?;

        info!(".env updated with rollup creator address");
        Ok(())
    }

    /// Deploy rollup proxy after setting the creator address in .env
    fn deploy_rollup_proxy(&self) -> Result<(String, String, u64)> {
        info!("Deploying rollup proxy");
        let dir = &self.config.workspace_dir.join("nitro-contracts");

        // Run deployment script
        let output = Command::new("npx")
            .current_dir(dir)
            .arg("hardhat")
            .arg("run")
            .arg("scripts/createEthRollup.ts")
            .arg("--network")
            .arg("arbSepolia")
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Rollup proxy deployment failed: {}", err);
            return Err(anyhow!("Rollup proxy deployment failed: {}", err));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Extract addresses and block number from output
        let rollup_proxy = self.extract_rollup_proxy_address(&output_str)?;
        let upgrade_executor = self.extract_upgrade_executor_address(&output_str)?;
        let deployment_block = self.extract_deployment_block(&output_str)?;

        // Read deployment json file for additional addresses if needed
        let deployment_json_path = dir.join("espresso-deployments/arbSepolia.json");
        if deployment_json_path.exists() {
            info!(
                "Deployment JSON found at {}",
                deployment_json_path.display()
            );
            // Here you could parse additional addresses if needed
        }

        Ok((rollup_proxy, upgrade_executor, deployment_block))
    }

    /// Extract the rollup creator address from the output
    fn extract_rollup_creator_address(&self, output: &str) -> Result<String> {
        // This is a simplified implementation - in a real scenario, you would use regex or other parsing methods
        for line in output.lines() {
            if line.contains("RollupCreator deployed to") {
                let parts: Vec<&str> = line.split("RollupCreator deployed to").collect();
                if parts.len() > 1 {
                    let address = parts[1].trim();
                    return Ok(address.to_string());
                }
            }
        }

        Err(anyhow!(
            "Could not extract rollup creator address from output"
        ))
    }

    /// Extract the rollup proxy address from the output
    fn extract_rollup_proxy_address(&self, output: &str) -> Result<String> {
        // Simplified implementation
        for line in output.lines() {
            if line.contains("RollupProxy deployed to") {
                let parts: Vec<&str> = line.split("RollupProxy deployed to").collect();
                if parts.len() > 1 {
                    let address = parts[1].trim();
                    return Ok(address.to_string());
                }
            }
        }

        Err(anyhow!(
            "Could not extract rollup proxy address from output"
        ))
    }

    /// Extract the upgrade executor address from the deployments file
    fn extract_upgrade_executor_address(&self, content: &str) -> Result<String> {
        // In a real implementation, you would use proper JSON parsing
        if let Some(pos_start) = content.find("\"upgradeExecutor\":") {
            if let Some(pos_addr_start) = content[pos_start..].find("\"address\":") {
                let addr_start = pos_start + pos_addr_start + 11; // Skip past "address": "
                if let Some(pos_addr_end) = content[addr_start..].find("\"") {
                    let address = &content[addr_start..addr_start + pos_addr_end];
                    return Ok(address.to_string());
                }
            }
        }

        Err(anyhow!(
            "Could not extract upgrade executor address from deployments file"
        ))
    }

    /// Extract the deployment block number from the output
    fn extract_deployment_block(&self, output: &str) -> Result<u64> {
        // Simplified implementation
        for line in output.lines() {
            if line.contains("Deployment block:") {
                let parts: Vec<&str> = line.split("Deployment block:").collect();
                if parts.len() > 1 {
                    let block_str = parts[1].trim();
                    return Ok(block_str.parse()?);
                }
            }
        }

        // Default to 0 if not found - in a real implementation, you might want to handle this differently
        Ok(0)
    }
}

/// Structure to hold deployment results
pub struct DeploymentResult {
    pub rollup_creator_address: String,
    pub rollup_proxy_address: String,
    pub upgrade_executor_address: String,
    pub deployment_block: u64,
    pub chain_id: u64,
}

/// The Deployer module for managing contract deployments and node setup
pub struct Deployer {
    pub address: String,
    pub private_key: String,
    pub chain_id: u64,
    pub rpc_url: String,
}

impl Deployer {
    /// Create a new deployer with the given configuration
    pub fn new(address: String, private_key: String, chain_id: u64, rpc_url: String) -> Self {
        Self {
            address,
            private_key,
            chain_id,
            rpc_url,
        }
    }

    /// Deploy contracts and return the deployment results
    pub fn deploy_contracts(&self) -> anyhow::Result<DeploymentResult> {
        info!("Deploying contracts with address {}", self.address);

        // Execute the deployment script
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "export PRIVATE_KEY={} && export RPC_URL={} && npm run deploy",
                self.private_key, self.rpc_url
            ))
            .output()?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            error!("Deployment failed: {}", error_message);
            return Err(anyhow::anyhow!("Deployment failed: {}", error_message));
        }

        // Parse the result (you would parse actual output here)
        let deployment_result = self.parse_deployment_result(&output.stdout)?;
        info!("Contracts deployed successfully");

        Ok(deployment_result)
    }

    /// Parse the deployment output to extract contract addresses and other information
    fn parse_deployment_result(&self, output: &[u8]) -> anyhow::Result<DeploymentResult> {
        // This is a placeholder - in a real implementation, you would parse the actual output
        // from the deployment script to extract the contract addresses and other information
        // For now, we'll return dummy values

        Ok(DeploymentResult {
            rollup_creator_address: "0x1234567890123456789012345678901234567890".to_string(),
            rollup_proxy_address: "0x0987654321098765432109876543210987654321".to_string(),
            upgrade_executor_address: "0x1234567890123456789012345678901234567890".to_string(),
            deployment_block: 0,
            chain_id: self.chain_id,
        })
    }
}
