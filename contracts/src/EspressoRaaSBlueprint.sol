// SPDX-License-Identifier: UNLICENSE
pragma solidity >=0.8.13;

import "tnt-core/BlueprintServiceManagerBase.sol";

/**
 * @title EspressoRaaSBlueprint
 * @dev This contract manages Espresso Rollup as a Service using Arbitrum Nitro Orbit.
 */
contract EspressoRaaSBlueprint is BlueprintServiceManagerBase {
    // Rollup configuration struct
    struct RollupConfig {
        uint256 chainId;
        address initialChainOwner;
        address[] validators;
        address batchPosterAddress;
        address batchPosterManager;
        bool isActive;
        uint256 createdAt;
        bool isMainnet; // Indicates if the rollup should be deployed to mainnet
    }

    // Mapping from serviceId to rollup configurations
    mapping(uint64 => RollupConfig) public rollups;

    // Events
    event RollupCreated(
        uint64 indexed serviceId,
        uint256 chainId,
        address initialChainOwner,
        bool isMainnet
    );
    event RollupStarted(uint64 indexed serviceId, uint256 chainId);
    event RollupStopped(uint64 indexed serviceId, uint256 chainId);

    // Job IDs
    uint8 constant CREATE_ROLLUP_JOB = 0;
    uint8 constant START_ROLLUP_JOB = 1;
    uint8 constant STOP_ROLLUP_JOB = 2;

    /**
     * @dev Hook for service operator registration.
     */
    function onRegister(
        ServiceOperators.OperatorPreferences calldata operator,
        bytes calldata registrationInputs
    ) external payable virtual override onlyFromMaster {
        // Validate that the operator has the necessary capabilities
        // For now, we don't need any specific validation
    }

    /**
     * @dev Hook for service instance requests.
     */
    function onRequest(
        ServiceOperators.RequestParams calldata params
    ) external payable virtual override onlyFromMaster {}

    /**
     * @dev Hook for handling job result.
     */
    function onJobResult(
        uint64 serviceId,
        uint8 job,
        uint64 jobCallId,
        ServiceOperators.OperatorPreferences calldata operator,
        bytes calldata inputs,
        bytes calldata outputs
    ) external payable virtual override onlyFromMaster {
        if (job == CREATE_ROLLUP_JOB) {
            // Extract rollup configuration from params
            (
                uint256 chainId,
                address initialChainOwner,
                address[] memory validators,
                address batchPosterAddress,
                address batchPosterManager,
                bool isMainnet
            ) = abi.decode(
                    inputs,
                    (uint256, address, address[], address, address, bool)
                );

            // If deploying to mainnet, ensure chainId is >= 1000000 to avoid conflicts
            if (isMainnet) {
                require(
                    chainId >= 1000000,
                    "Mainnet chain IDs must be >= 1000000"
                );
            } else {
                require(
                    chainId < 1000000,
                    "Testnet chain IDs must be < 1000000"
                );
            }

            // Store rollup configuration
            rollups[serviceId] = RollupConfig({
                chainId: chainId,
                initialChainOwner: initialChainOwner,
                validators: validators,
                batchPosterAddress: batchPosterAddress,
                batchPosterManager: batchPosterManager,
                isActive: false,
                createdAt: block.timestamp,
                isMainnet: isMainnet
            });

            // Emit event
            emit RollupCreated(
                serviceId,
                chainId,
                initialChainOwner,
                isMainnet
            );
            // Handle rollup creation result
            bool success = abi.decode(outputs, (bool));
            if (success) {
                rollups[serviceId].isActive = true;
                emit RollupStarted(serviceId, rollups[serviceId].chainId);
            }
        } else if (job == START_ROLLUP_JOB) {
            // Handle rollup start result
            bool success = abi.decode(outputs, (bool));
            if (success) {
                rollups[serviceId].isActive = true;
                emit RollupStarted(serviceId, rollups[serviceId].chainId);
            }
        } else if (job == STOP_ROLLUP_JOB) {
            // Handle rollup stop result
            bool success = abi.decode(outputs, (bool));
            if (success) {
                rollups[serviceId].isActive = false;
                emit RollupStopped(serviceId, rollups[serviceId].chainId);
            }
        }
    }

    /**
     * @dev Converts a public key to an operator address.
     */
    function operatorAddressFromPublicKey(
        bytes calldata publicKey
    ) internal pure returns (address operator) {
        return address(uint160(uint256(keccak256(publicKey))));
    }
}
