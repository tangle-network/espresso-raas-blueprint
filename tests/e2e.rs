use blueprint_sdk as sdk;
use espresso_raas_blueprint::docker::{
    create_docker_rollup, delete_docker_rollup, start_docker_rollup, stop_docker_rollup,
};
use espresso_raas_blueprint::{RollupConfigParams, ServiceContext};

use sdk::Job;
use sdk::tangle::layers::TangleLayer;
use sdk::tangle::serde::{from_field, to_field};
use sdk::testing::{tempfile, utils::*};

#[tokio::test]
async fn test_rollup_creation() -> color_eyre::Result<()> {
    setup_log();

    // Initialize test harness (node, keys, deployment)
    let temp_dir = tempfile::TempDir::new()?;
    let harness = tangle::TangleTestHarness::setup(temp_dir).await?;

    // Setup service
    let (mut test_env, service_id, _) = harness.setup_services::<1>(false).await?;
    test_env.initialize().await?;

    // Register the job handlers for Docker rollups
    let handles = test_env.node_handles().await;
    let mut contexts = Vec::new();
    for handle in handles {
        let config = handle.gadget_config().await;

        // Create a context for the jobs
        let context = ServiceContext {
            config: config.clone(),
        };

        // Register each job handler
        handle
            .add_job(create_docker_rollup.layer(TangleLayer))
            .await;
        handle.add_job(start_docker_rollup.layer(TangleLayer)).await;
        handle.add_job(stop_docker_rollup.layer(TangleLayer)).await;
        handle
            .add_job(delete_docker_rollup.layer(TangleLayer))
            .await;

        contexts.push(context);
    }

    // Start the test environment
    test_env.start_with_contexts(contexts).await?;

    // Create a sample rollup configuration
    let rollup_config = RollupConfigParams {
        chain_id: 42,
        initial_chain_owner: "0x123456789abcdef0123456789abcdef012345678".to_string(),
        validators: vec![
            "0xabcdef0123456789abcdef0123456789abcdef01".to_string(),
            "0x9876543210abcdef9876543210abcdef98765432".to_string(),
        ],
        batch_poster_address: "0x2468ace02468ace02468ace02468ace02468ace0".to_string(),
        batch_poster_manager: "0x1357bdf91357bdf91357bdf91357bdf91357bdf9".to_string(),
        is_mainnet: false,
    };

    // Serialize the config for the job input
    let config_bytes = serde_json::to_vec(&rollup_config)?;

    // Submit the create_docker_rollup job (job ID 3)
    let job_inputs = vec![to_field(config_bytes).unwrap()];
    let job = harness.submit_job(service_id, 0, job_inputs).await?;

    // Wait for job execution and verify success
    let results = harness.wait_for_job_execution(service_id, job).await?;

    // The job should return a boolean success value as true
    assert_eq!(results.service_id, service_id);

    // Expecting a successful creation (true)
    let success_field = results.result[0].clone();
    let success: bool = from_field(success_field)?;
    assert!(success, "Rollup creation should succeed");

    Ok(())
}
