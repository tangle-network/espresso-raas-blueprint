use blueprint_sdk as sdk;

use crate::RollupConfig;
use anyhow::{Result, anyhow};
use sdk::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
    pub network: String,
    pub initial_chain_owner: [u8; 20],
    pub validators: Vec<[u8; 20]>,
    pub batch_poster_address: [u8; 20],
    pub batch_poster_manager: [u8; 20],
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
            initial_chain_owner: rollup_config.initial_chain_owner,
            validators: rollup_config.validators.to_vec(),
            batch_poster_address: rollup_config.batch_poster_address,
            batch_poster_manager: rollup_config.batch_poster_manager,
            network: rollup_config.network.to_string(),
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

        // First check if we're on develop branch
        info!("Verifying we're on the develop branch");
        let output = Command::new("git")
            .current_dir(dir)
            .args(["branch", "--show-current"])
            .output()?;

        let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Current branch: {}", current_branch);
        if current_branch != NITRO_CONTRACTS_BRANCH {
            error!(
                "Not on {} branch, currently on {}",
                NITRO_CONTRACTS_BRANCH, current_branch
            );
            return Err(anyhow!("Not on {} branch", NITRO_CONTRACTS_BRANCH));
        }

        // Run yarn install for package dependencies
        info!("Installing yarn dependencies");
        self.run_command("yarn", &["install"], dir)?;

        info!("Installing forge dependencies");
        self.run_command("forge", &["install"], dir)?;

        info!("Building contracts with yarn build:all");
        self.run_command("yarn", &["build:all"], dir)?;

        info!("Contracts built successfully");

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
             IGNORE_MAX_DATA_SIZE_WARNING=true\n\
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
            .replace(
                "OWNER_ADDRESS",
                &hex::encode(self.config.initial_chain_owner),
            )
            .replace("YOUR_CHAIN_ID", &self.config.chain_id.to_string())
            .replace("ChainID", &self.config.chain_id.to_string())
            .replace(
                "YOUR_OWNED_ADDRESS",
                &hex::encode(self.config.initial_chain_owner),
            )
            .replace("AN_OWNED_ADDRESS", &hex::encode(self.config.validators[0]))
            .replace(
                "ANOTHER_OWNED_ADDRESS",
                &hex::encode(self.config.batch_poster_address),
            );

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
            .arg(&self.config.network)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Deployment failed: {}", err);
            return Err(anyhow!("Deployment failed: {}", err));
        }

        // Extract rollup creator address from output
        let deployments = dir.join("espresso-deployments");
        let deployment_json = deployments.join(format!("{}.json", self.config.network));
        if deployment_json.exists() {
            info!("Deployment JSON found at {}", deployment_json.display());
        } else {
            error!("Deployment JSON not found at {}", deployment_json.display());
            return Err(anyhow!("Deployment JSON not found"));
        }
        // Read the file and extract the rollup creator address
        let output_json = fs::read_to_string(&deployment_json)?;
        let output = serde_json::from_str::<serde_json::Value>(&output_json)?;
        self.extract_rollup_creator_address(&output)
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

        // Read deployment json file for additional addresses if needed
        let deployment_json_path = dir
            .join("espresso-deployments")
            .join(format!("{}.json", self.config.network));
        if !deployment_json_path.exists() {
            error!(
                "Deployment JSON not found at {}",
                deployment_json_path.display()
            );
            return Err(anyhow!("Deployment JSON not found"));
        }

        let deployment_json = fs::read_to_string(&deployment_json_path)?;
        let deployment = serde_json::from_str::<serde_json::Value>(&deployment_json)?;

        // Run deployment script
        let output = Command::new("npx")
            .current_dir(dir)
            .arg("hardhat")
            .arg("run")
            .arg("scripts/createEthRollup.ts")
            .arg("--network")
            .arg(&self.config.network)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Rollup proxy deployment failed: {}", err);
            return Err(anyhow!("Rollup proxy deployment failed: {}", err));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Extract addresses and block number from output
        let upgrade_executor = self.extract_upgrade_executor_address(&deployment)?;
        let rollup_proxy = self.extract_rollup_proxy_address(&output_str)?;
        let deployment_block = self.extract_deployment_block(&output_str)?;

        Ok((rollup_proxy, upgrade_executor, deployment_block))
    }

    /// Extract the rollup creator address from the output
    fn extract_rollup_creator_address(&self, output: &serde_json::Value) -> Result<String> {
        output
            .get("RollupCreator")
            .and_then(|creator| creator.as_str())
            .map(|address| address.to_string())
            .ok_or_else(|| anyhow!("Could not extract rollup creator address from output"))
    }

    /// Extract the rollup proxy address from the output
    fn extract_rollup_proxy_address(&self, output: &str) -> Result<String> {
        // Simplified implementation
        for line in output.lines() {
            if line.contains("RollupProxy Contract created at address:") {
                let parts: Vec<&str> = line
                    .split("RollupProxy Contract created at address:")
                    .collect();
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
    fn extract_upgrade_executor_address(&self, content: &serde_json::Value) -> Result<String> {
        content
            .get("UpgradeExecutor")
            .and_then(|executor| executor.as_str())
            .map(|address| address.to_string())
            .ok_or_else(|| {
                anyhow!("Could not extract upgrade executor address from deployments file")
            })
    }

    /// Extract the deployment block number from the output
    fn extract_deployment_block(&self, output: &str) -> Result<u64> {
        // Simplified implementation
        for line in output.lines() {
            if line.contains("All deployed at block number:") {
                let parts: Vec<&str> = line.split("All deployed at block number:").collect();
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
