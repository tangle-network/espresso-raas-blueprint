use anyhow::Result;
use blueprint_sdk::logging;
use clap::Parser;
use espresso_raas_blueprint::list_rollups;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logging::setup_log();

    // Log the operation
    logging::info!("Listing all rollups");

    // List all rollups
    let rollups = list_rollups().await;

    // Display rollup information
    logging::info!("Found {} rollups", rollups.len());
    for (i, rollup) in rollups.iter().enumerate() {
        logging::info!("Rollup #{}", i + 1);
        for (key, value) in rollup {
            logging::info!("  {}: {}", key, value);
        }
    }

    Ok(())
}
