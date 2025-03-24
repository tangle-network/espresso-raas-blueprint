// SPDX-License-Identifier: UNLICENSE
pragma solidity >=0.8.13;

import "tnt-core/BlueprintServiceManagerBase.sol";

/**
 * @title EspressoRaaSBlueprint
 * @dev This contract manages Espresso Rollup as a Service using Arbitrum Nitro Orbit.
 */
contract EspressoRaaSBlueprint is BlueprintServiceManagerBase {
    /**
     * @dev Converts a public key to an operator address.
     */
    function operatorAddressFromPublicKey(
        bytes calldata publicKey
    ) internal pure returns (address operator) {
        return address(uint160(uint256(keccak256(publicKey))));
    }
}
