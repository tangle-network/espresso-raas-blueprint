use anyhow::Result;
use blueprint_sdk::{
    alloy::{primitives::Address, signers::local::PrivateKeySigner},
    logging,
};
use clap::Parser;
use dotenv::dotenv;
use espresso_raas_blueprint::{create_rollup, RollupConfig, RollupConfigParams};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Chain ID for the rollup
    #[arg(short, long, default_value_t = 42)]
    chain_id: u64,

    /// Owner address for the rollup
    #[arg(
        short,
        long,
        default_value = "0x1234567890123456789012345678901234567890"
    )]
    owner: String,

    /// Validator address for the rollup
    #[arg(
        short,
        long,
        default_value = "0x2345678901234567890123456789012345678901"
    )]
    validator: String,

    /// Batch poster address for the rollup
    #[arg(
        short = 'p',
        long,
        default_value = "0x3456789012345678901234567890123456789012"
    )]
    batch_poster: String,

    /// Batch manager address for the rollup
    #[arg(
        short = 'm',
        long,
        default_value = "0x4567890123456789012345678901234567890123"
    )]
    batch_manager: String,

    /// Use mainnet instead of testnet
    #[arg(short = 'n', long)]
    mainnet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Initialize logging
    logging::setup_log();

    // Parse command line arguments
    let args = Args::parse();

    let deployer_key = std::env::var("DEPLOYER_PRIVATE_KEY").unwrap();
    let validator_key = std::env::var("VALIDATOR_PRIVATE_KEY").unwrap();
    let batch_poster_key = std::env::var("BATCH_POSTER_PRIVATE_KEY").unwrap();

    let deployer: PrivateKeySigner = deployer_key.parse().unwrap();
    let validator: PrivateKeySigner = validator_key.parse().unwrap();
    let batch_poster: PrivateKeySigner = batch_poster_key.parse().unwrap();

    // Create rollup configuration
    let config = RollupConfigParams {
        chain_id: args.chain_id,
        initial_chain_owner: deployer.address().to_string(),
        validators: vec![validator.address().to_string()],
        batch_poster_address: batch_poster.address().to_string(),
        batch_poster_manager: batch_poster.address().to_string(),
        is_mainnet: args.mainnet,
    };

    // Log configuration details
    logging::info!("Creating rollup with configuration:");
    logging::info!("  Chain ID: {}", args.chain_id);
    logging::info!("  Owner: {}", deployer.address().to_string());
    logging::info!("  Validator: {}", validator.address().to_string());
    logging::info!("  Batch Poster: {}", batch_poster.address().to_string());
    logging::info!("  Batch Manager: {}", batch_poster.address().to_string());
    logging::info!(
        "  Network: {}",
        if args.mainnet { "Mainnet" } else { "Testnet" }
    );

    // Simulate job execution
    let service_id = 1; // In a real scenario, this would come from the context
    logging::info!("Executing create_rollup job for service_id: {}", service_id);

    // Generate a unique rollup_id
    let rollup_id = Uuid::new_v4().to_string();
    logging::info!("Generated rollup_id: {}", rollup_id);

    // Generate a unique VM ID
    let vm_id = format!("rollup-{}-{}", service_id, Uuid::new_v4());
    logging::info!("Generated VM ID: {}", vm_id);

    // Create the rollup
    match create_rollup(
        service_id,
        rollup_id.clone(),
        vm_id.clone(),
        RollupConfig::from(config),
    )
    .await
    {
        Ok(_) => {
            logging::info!("Rollup created successfully!");
            logging::info!("Service ID: {}", service_id);
            logging::info!("Rollup ID: {}", rollup_id);
            logging::info!("VM ID: {}", vm_id);
        }
        Err(e) => {
            logging::error!("Failed to create rollup: {}", e);
        }
    }

    Ok(())
}
