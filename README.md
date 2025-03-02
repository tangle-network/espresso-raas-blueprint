# Espresso RaaS Blueprint

A Tangle Blueprint for Espresso Rollup as a Service (RaaS) using Arbitrum Nitro Orbit and Docker.

## Overview

This blueprint provides a service for managing Espresso rollups using Arbitrum Nitro Orbit. It allows operators to create, start, stop, and monitor rollups through smart contracts and the Tangle Blueprint SDK.

The system uses Docker to isolate and manage multiple rollups, providing a secure and scalable solution for rollup deployment.

## Architecture

The system consists of the following components:

1. **Smart Contract**: The `EspressoRaaSBlueprint` contract manages rollup configurations and handles job requests.
2. **Rust SDK**: The Rust implementation handles events from the smart contract and manages Docker.
3. **Docker**: Each rollup runs in its own Docker container, providing isolation and security.
4. **Arbitrum Nitro Orbit**: The rollup technology used for creating and managing rollups.

## Prerequisites

- Rust 1.81 or later
- Docker and Docker Compose
- Access to Arbitrum Sepolia or Arbitrum One network

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/tangle-network/espresso-raas-blueprint.git
   cd espresso-raas-blueprint
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

## Development

### Building from Source

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
