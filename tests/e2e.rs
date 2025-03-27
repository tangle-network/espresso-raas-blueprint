use blueprint_sdk as sdk;
use blueprint_sdk::testing::utils::tangle::harness::SetupServicesOpts;
use espresso_raas_blueprint::docker::{
    create_docker_rollup, delete_docker_rollup, start_docker_rollup, stop_docker_rollup,
};
use espresso_raas_blueprint::{NetworkType, RollupConfigParams, ServiceContext};
use hex_literal::hex;

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

    // Create a sample rollup configuration
    let rollup_config = RollupConfigParams {
        chain_id: 42,
        initial_chain_owner: hex!("123456789abcdef0123456789abcdef012345678"),
        validators: vec![
            hex!("abcdef0123456789abcdef0123456789abcdef01"),
            hex!("9876543210abcdef9876543210abcdef98765432"),
        ]
        .into(),
        batch_poster_address: hex!("2468ace02468ace02468ace02468ace02468ace0"),
        batch_poster_manager: hex!("1357bdf91357bdf91357bdf91357bdf91357bdf9"),
        is_mainnet: false,
        network: NetworkType::Geth,
    };
    // Setup service
    let (mut test_env, service_id, _) = harness
        .setup_services_with_options::<1>(SetupServicesOpts {
            exit_after_registration: false,
            ..Default::default()
        })
        .await?;
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

    // Submit the create_docker_rollup job (job ID 3)
    let job_inputs = vec![to_field(rollup_config.clone()).unwrap()];
    let job = harness.submit_job(service_id, 0, job_inputs).await?;

    // Wait for job execution and verify success
    let results = harness.wait_for_job_execution(service_id, job).await?;

    // The job should return a boolean success value as true
    assert_eq!(results.service_id, service_id);

    // Expecting a successful creation of the rollup
    let success_field = results.result[0].clone();
    let rollup_id: String = from_field(success_field)?;
    println!("Rollup ID: {}", rollup_id);

    // Start the rollup
    let job_inputs = vec![to_field(rollup_id.clone()).unwrap()];
    let job = harness.submit_job(service_id, 1, job_inputs).await?;
    let results = harness.wait_for_job_execution(service_id, job).await?;
    assert_eq!(results.service_id, service_id);

    Ok(())
}
