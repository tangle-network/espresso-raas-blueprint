1. Create keystore directory (`mkdir -p ./target/keystore`)
2. Insert Some keys into the keystore using tangle cli:

```bash
cargo tangle key import -t sr25519 -k target/keystore -x e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a # Alice
cargo tangle key import -t ecdsa -k target/keystore -x cb6df9de1efca7a3998a8ead4e02159d5fa99c3e0d4fd6432667390bb4726854 # Alice (ECDSA)
```

3. Build contracts (if not already built):

```bash
forge build
```

3. Run the tangle network locally(or use the devnet):

```bash
./target/release/tangle --tmp --dev --validator -linfo --alice --rpc-cors all --rpc-methods=unsafe --rpc-external --rpc-port 9944 -levm=debug -lgadget=trace --sealing manual
```

4. Deploy the MBSM to the network:

```bash
cargo tangle blueprint deploy-mbsm
```

5. Build the blueprint:

```bash
cargo build --workspace
```

6. Deploy the blueprint:

```bash
cargo tangle blueprint deploy tangle --http-rpc-url http://127.0.0.1:9944 --ws-rpc-url ws://127.0.0.1:9944 -k target/keystore
```

7. Verify the deployment:

```bash
cargo tangle blueprint lb
```

8. Register on the blueprint as an operator:

```bash
cargo tangle blueprint register --blueprint-id 0 --keystore-uri ./target/keystore
```

9. Request a new service instance:

```bash
cargo tangle blueprint request-service --blueprint-id 0 --keystore-uri ./target/keystore --value 0 --target-operators 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

10. Approve the service instance:

```bash
cargo tangle blueprint accept-request --request-id 0 --keystore-uri ./target/keystore
```

11. Run the blueprint:

```bash
cargo tangle blueprint run --protocol tangle -k target/keystore
```

Enter the blueprint id and service instance id when prompted. (`0` and `0` respectively)
