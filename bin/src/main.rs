use blueprint_sdk as sdk;
use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;

use anyhow::Result;
use espresso_raas_blueprint as blueprint;
use sdk::contexts::tangle::TangleClientContext;
use sdk::crypto::sp_core::SpSr25519;
use sdk::crypto::tangle_pair_signer::TanglePairSigner;
use sdk::keystore::backends::Backend;
use sdk::runner::BlueprintRunner;
use sdk::runner::config::BlueprintEnvironment;
use sdk::runner::tangle::config::TangleConfig;
use sdk::tangle::consumer::TangleConsumer;
use sdk::tangle::producer::TangleProducer;
use tower::filter::FilterLayer;

#[tokio::main]
async fn main() -> Result<()> {
    setup_log();

    let env = BlueprintEnvironment::load()?;
    let config = TangleConfig::new(Default::default());

    // Signer
    let sr25519_signer = env.keystore().first_local::<SpSr25519>()?;
    let sr25519_pair = env.keystore().get_secret::<SpSr25519>(&sr25519_signer)?;
    let st25519_signer = TanglePairSigner::new(sr25519_pair.0);

    // Producer
    let tangle_client = env.tangle_client().await?;
    let tangle_producer = TangleProducer::best_blocks(tangle_client.rpc_client.clone()).await?;
    // Consumer
    let tangle_consumer = TangleConsumer::new(tangle_client.rpc_client.clone(), st25519_signer);

    let service_id = env.protocol_settings.tangle()?.service_id.unwrap();
    let context = blueprint::ServiceContext::new(env.clone());
    let router = sdk::Router::new()
        .route(0, blueprint::docker::jobs::create_docker_rollup)
        .route(1, blueprint::docker::jobs::start_docker_rollup)
        .route(2, blueprint::docker::jobs::stop_docker_rollup)
        .route(3, blueprint::docker::jobs::delete_docker_rollup)
        .layer(TangleLayer)
        .layer(FilterLayer::new(MatchesServiceId(service_id)))
        .with_context(context);
    sdk::info!("Starting the event watcher ...");
    let result = BlueprintRunner::builder(config, env)
        .router(router)
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .run()
        .await;
    if let Err(e) = result {
        sdk::error!("Runner failed! {e:?}");
    }
    Ok(())
}

pub fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::metadata::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
