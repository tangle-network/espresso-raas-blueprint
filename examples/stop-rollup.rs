use anyhow::Result;
use blueprint_sdk::logging;
use clap::Parser;
use espresso_raas_blueprint::stop_rollup;

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
    logging::info!("Stopping rollup for service_id: {}", args.service_id);

    // Stop the rollup
    match stop_rollup(args.service_id).await {
        Ok(_) => {
            logging::info!("Rollup stopped successfully!");
            logging::info!("Service ID: {}", args.service_id);
        }
        Err(e) => {
            logging::error!("Failed to stop rollup: {}", e);
        }
    }

    Ok(())
}
