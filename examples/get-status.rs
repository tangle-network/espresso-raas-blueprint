use anyhow::Result;
use blueprint_sdk::logging;
use clap::Parser;
use espresso_raas_blueprint::get_rollup_status;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// VM ID of the rollup to check
    #[arg(short, long)]
    vm_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    logging::setup_log();

    // Parse command line arguments
    let args = Args::parse();

    // Log the VM ID
    logging::info!("Getting status for rollup with VM ID: {}", args.vm_id);

    // Get the status
    match get_rollup_status(&args.vm_id).await {
        Ok(status) => {
            logging::info!("Rollup status: {}", status);
            logging::info!("VM ID: {}", args.vm_id);
        }
        Err(e) => {
            logging::error!("Failed to get rollup status: {}", e);
        }
    }

    Ok(())
}
