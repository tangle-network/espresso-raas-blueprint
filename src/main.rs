use blueprint_sdk::logging;
use blueprint_sdk::runners::core::runner::BlueprintRunner;
use blueprint_sdk::runners::tangle::tangle::TangleConfig;
use espresso_raas_blueprint as blueprint;

#[blueprint_sdk::main(env)]
async fn main() {
    // Create your service context
    // Here you can pass any configuration or context that your service needs.
    let context = blueprint::ServiceContext {
        config: env.clone(),
        call_id: None,
        service_id: env.protocol_settings.tangle().unwrap().service_id.unwrap(),
    };

    // Create the event handlers for our jobs

    logging::info!("Starting the Espresso RaaS Blueprint...");

    // Configure and run the blueprint runner with our jobs
    let tangle_config = TangleConfig::default();
    BlueprintRunner::new(tangle_config, env).run().await?;

    logging::info!("Exiting...");
    Ok(())
}
