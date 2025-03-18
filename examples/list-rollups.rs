use anyhow::Result;
use blueprint_sdk as sdk;
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
    sdk::info!("Listing all rollups");

    // List all rollups
    let rollups = list_rollups().await;

    // Display rollup information
    sdk::info!("Found {} rollups", rollups.len());
    for (i, rollup) in rollups.iter().enumerate() {
        sdk::info!("Rollup #{}", i + 1);
        for (key, value) in rollup {
            sdk::info!("  {}: {}", key, value);
        }
    }

    Ok(())
}
