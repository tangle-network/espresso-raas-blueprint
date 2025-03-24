use blueprint_sdk as sdk;
use blueprint_sdk::tangle::extract::List;

use sdk::macros::context::{ServicesContext, TangleClientContext};
use sdk::runner::config::BlueprintEnvironment;
use serde::{Deserialize, Serialize};

pub mod deployer;
pub mod docker;

pub use deployer::DeploymentResult;

// Re-export Docker functionality
pub use docker::{
    EspressoDockerManager, RollupInfo, RollupManager, RollupStatus as DockerRollupStatus,
    create_rollup, delete_rollup, get_rollup_status, list_rollups, start_rollup, stop_rollup,
};

// Service context for our blueprint
#[derive(Clone, TangleClientContext, ServicesContext)]
pub struct ServiceContext {
    #[config]
    pub config: BlueprintEnvironment,
}

impl ServiceContext {
    pub fn new(config: BlueprintEnvironment) -> Self {
        Self { config }
    }
}

/// Network type for the rollup
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NetworkType {
    ArbitrumMainnet,
    ArbitrumSepolia,
}

impl NetworkType {
    pub fn rpc_url(&self) -> &str {
        match self {
            NetworkType::ArbitrumMainnet => "https://arb1.arbitrum.io/rpc",
            NetworkType::ArbitrumSepolia => "https://sepolia-rollup.arbitrum.io/rpc",
        }
    }

    pub fn parent_chain_id(&self) -> u64 {
        match self {
            NetworkType::ArbitrumMainnet => 1,        // Ethereum Mainnet
            NetworkType::ArbitrumSepolia => 11155111, // Ethereum Sepolia
        }
    }
}

// Serializable rollup configuration for job parameters
#[derive(Default, Serialize, Deserialize)]
pub struct RollupConfigParams {
    /// Chain ID
    pub chain_id: u64,
    /// Initial chain owner
    pub initial_chain_owner: [u8; 20],
    /// Validators
    pub validators: List<[u8; 20]>,
    /// Batch poster address
    pub batch_poster_address: [u8; 20],
    /// Batch poster manager
    pub batch_poster_manager: [u8; 20],
    /// Is mainnet
    pub is_mainnet: bool,
}

impl std::fmt::Debug for RollupConfigParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RollupConfigParams")
            .field("chain_id", &self.chain_id)
            .field(
                "initial_chain_owner",
                &hex::encode(self.initial_chain_owner),
            )
            .field(
                "validators",
                &self
                    .validators
                    .0
                    .iter()
                    .map(hex::encode)
                    .collect::<Vec<_>>(),
            )
            .field(
                "batch_poster_address",
                &hex::encode(self.batch_poster_address),
            )
            .field(
                "batch_poster_manager",
                &hex::encode(self.batch_poster_manager),
            )
            .field("is_mainnet", &self.is_mainnet)
            .finish()
    }
}

impl Clone for RollupConfigParams {
    fn clone(&self) -> Self {
        Self {
            chain_id: self.chain_id,
            initial_chain_owner: self.initial_chain_owner,
            validators: List(self.validators.0.clone()),
            batch_poster_address: self.batch_poster_address,
            batch_poster_manager: self.batch_poster_manager,
            is_mainnet: self.is_mainnet,
        }
    }
}

/// Rollup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupConfig {
    /// Chain ID
    pub chain_id: u64,
    /// Initial chain owner
    pub initial_chain_owner: [u8; 20],
    /// Validators
    pub validators: Vec<[u8; 20]>,
    /// Batch poster address
    pub batch_poster_address: [u8; 20],
    /// Batch poster manager
    pub batch_poster_manager: [u8; 20],
    /// Is mainnet
    pub network: NetworkType,
}

/// Convert RollupConfigParams to RollupConfig
impl From<RollupConfigParams> for RollupConfig {
    fn from(params: RollupConfigParams) -> Self {
        Self {
            chain_id: params.chain_id,
            initial_chain_owner: params.initial_chain_owner,
            validators: params.validators.0,
            batch_poster_address: params.batch_poster_address,
            batch_poster_manager: params.batch_poster_manager,
            network: if params.is_mainnet {
                NetworkType::ArbitrumMainnet
            } else {
                NetworkType::ArbitrumSepolia
            },
        }
    }
}
