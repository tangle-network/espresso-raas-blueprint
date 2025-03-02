use anyhow::Result;
use blueprint_sdk::logging;
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

    // Create rollup configuration
    let config = RollupConfigParams {
        chain_id: args.chain_id,
        initial_chain_owner: args.owner.clone(),
        validators: vec![args.validator.clone()],
        batch_poster_address: args.batch_poster.clone(),
        batch_poster_manager: args.batch_manager.clone(),
        is_mainnet: args.mainnet,
    };

    // Log configuration details
    logging::info!("Creating rollup with configuration:");
    logging::info!("  Chain ID: {}", args.chain_id);
    logging::info!("  Owner: {}", args.owner);
    logging::info!("  Validator: {}", args.validator);
    logging::info!("  Batch Poster: {}", args.batch_poster);
    logging::info!("  Batch Manager: {}", args.batch_manager);
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
