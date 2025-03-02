use anyhow::Result;
use std::path::PathBuf;

pub mod config;
pub mod rollup;

// Re-export important types
pub use config::ConfigGenerator;
pub use rollup::RollupDeployer;

/// Structure to hold deployment results
#[derive(Clone)]
pub struct DeploymentResult {
    pub rollup_creator_address: String,
    pub rollup_proxy_address: String,
    pub upgrade_executor_address: String,
    pub deployment_block: u64,
    pub chain_id: u64,
}
