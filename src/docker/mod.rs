pub mod container;
pub mod espresso;
pub mod helpers;
pub mod jobs;
pub mod rollup;

// Re-export public types from container
pub use container::DockerComposeManager;

// Re-export helper functions
pub use helpers::{
    create_rollup, delete_rollup, get_rollup_status, list_rollups, start_rollup, stop_rollup,
};

// Re-export rollup types
pub use espresso::EspressoDockerManager;
pub use rollup::{RollupInfo, RollupManager, RollupStatus};

// Reexport from jobs
pub use jobs::{
    create_docker_rollup, delete_docker_rollup, start_docker_rollup, stop_docker_rollup,
};
