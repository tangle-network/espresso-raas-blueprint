use anyhow::Result;
use blueprint_sdk::logging;
use clap::Parser;
use espresso_raas_blueprint::start_rollup;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Service ID for the rollup
    #[arg(short, long, default_value_t = 1)]
    service_id: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logging::setup_log();

    // Parse command line arguments
    let args = Args::parse();

    // Log operation
    logging::info!("Starting rollup for service_id: {}", args.service_id);

    // Start the rollup
    match start_rollup(args.service_id).await {
        Ok(_) => {
            logging::info!("Rollup started successfully!");
            logging::info!("Service ID: {}", args.service_id);
        }
        Err(e) => {
            logging::error!("Failed to start rollup: {}", e);
        }
    }

    Ok(())
}
