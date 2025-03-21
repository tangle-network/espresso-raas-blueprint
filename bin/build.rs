use blueprint_sdk::build;
use blueprint_sdk::tangle::blueprint;
use espresso_raas_blueprint as blueprint;
use std::path::Path;
use std::process;

use blueprint::docker::jobs::{
    create_docker_rollup, delete_docker_rollup, start_docker_rollup, stop_docker_rollup,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract_dirs: Vec<&str> = vec!["../contracts"];
    build::utils::soldeer_install();
    build::utils::soldeer_update();
    build::utils::build_contracts(contract_dirs);

    println!("cargo::rerun-if-changed=../src");
    println!("cargo::rerun-if-changed=./src");

    let blueprint = blueprint! {
        name: "esoresso-raas-blueprint",
        master_manager_revision: "Latest",
        manager: { Evm = "EspressoRaaSBlueprint" },
        jobs: [
            create_docker_rollup,
            start_docker_rollup,
            stop_docker_rollup,
            delete_docker_rollup,
        ],
    };

    match blueprint {
        Ok(blueprint) => {
            let json = serde_json::to_string_pretty(&blueprint)?;
            std::fs::write(Path::new("../").join("blueprint.json"), json.as_bytes())?;
        }
        Err(e) => {
            println!("cargo::error={e:?}");
            process::exit(1);
        }
    }

    Ok(())
}
