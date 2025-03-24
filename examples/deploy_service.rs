use std::fs;

use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::crypto::k256::K256Ecdsa;
use blueprint_sdk::crypto::sp_core::SpSr25519;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::debug;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::testing::chain_setup::tangle::deploy::{Opts, deploy_to_tangle};
use blueprint_sdk::testing::chain_setup::tangle::transactions::*;
use blueprint_sdk::testing::utils::setup_log;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    setup_log();
    let env = BlueprintEnvironment::load()?;
    let keystore = env.keystore();
    let signer_pubkey = keystore.first_local::<SpSr25519>()?;
    let signer = keystore.get_secret::<SpSr25519>(&signer_pubkey)?;

    let evm_signer_pubkey = keystore.first_local::<K256Ecdsa>()?;
    let evm_signer = keystore.get_secret::<K256Ecdsa>(&evm_signer_pubkey)?;

    let opts = Opts {
        pkg_name: None,
        http_rpc_url: env.http_rpc_endpoint.clone(),
        ws_rpc_url: env.ws_rpc_endpoint.clone(),
        manifest_path: fs::canonicalize("Cargo.toml")?,
        signer: Some(TanglePairSigner::new(signer.0)),
        signer_evm: Some(evm_signer.alloy_key()?),
    };
    let tangle_client = env.tangle_client().await?;
    let latest_revision = get_latest_mbsm_revision(&tangle_client).await?;

    if let Some((rev, addr)) = latest_revision {
        debug!("MBSM is deployed at revision #{rev} at address {addr}");
    } else {
        debug!("MBSM is not deployed");

        let bytecode = tnt_core_bytecode::bytecode::MASTER_BLUEPRINT_SERVICE_MANAGER;
        deploy_new_mbsm_revision(
            &opts.ws_rpc_url,
            &tangle_client,
            opts.signer.as_ref().unwrap(),
            evm_signer.alloy_key()?,
            bytecode,
            [0u8; 20].into(),
        )
        .await?;
    }

    let blueprint_id = deploy_to_tangle(opts).await?;
    println!("Deployed blueprint with ID: {:?}", blueprint_id);
    Ok(())
}
