// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/ZkIPFSVerifier.sol";

/**
 * @title Deploy
 * @author Sowad Al-Mughni
 * @notice Deployment script for ZkIPFS verifier contracts
 */
contract Deploy is Script {
    /// @notice Default fee recipient (can be overridden)
    address public constant DEFAULT_FEE_RECIPIENT = 0x742d35Cc6634C0532925a3b8D4C9db96590c6C87;

    /// @notice Deploy the ZkIPFS verifier contract
    function run() external {
        // Get deployment parameters from environment or use defaults
        address owner = vm.envOr("OWNER_ADDRESS", msg.sender);
        address feeRecipient = vm.envOr("FEE_RECIPIENT", DEFAULT_FEE_RECIPIENT);
        uint256 verificationFee = vm.envOr("VERIFICATION_FEE", uint256(0));
        uint256 minSecurityLevel = vm.envOr("MIN_SECURITY_LEVEL", uint256(128));

        console.log("Deploying ZkIPFS Verifier...");
        console.log("Owner:", owner);
        console.log("Fee Recipient:", feeRecipient);
        console.log("Verification Fee:", verificationFee);
        console.log("Min Security Level:", minSecurityLevel);

        vm.startBroadcast();

        // Deploy the main verifier contract
        ZkIPFSVerifier verifier = new ZkIPFSVerifier(owner, feeRecipient);

        // Configure initial parameters if different from defaults
        if (verificationFee != 0) {
            verifier.setVerificationFee(verificationFee);
        }

        if (minSecurityLevel != 128) {
            verifier.setMinSecurityLevel(minSecurityLevel);
        }

        vm.stopBroadcast();

        console.log("ZkIPFS Verifier deployed at:", address(verifier));
        
        // Save deployment info
        _saveDeploymentInfo(address(verifier), owner, feeRecipient);
    }

    /// @notice Deploy with custom parameters
    function deployWithParams(
        address owner,
        address feeRecipient,
        uint256 verificationFee,
        uint256 minSecurityLevel,
        uint256 maxProofAge
    ) external {
        vm.startBroadcast();

        ZkIPFSVerifier verifier = new ZkIPFSVerifier(owner, feeRecipient);

        // Configure parameters
        verifier.setVerificationFee(verificationFee);
        verifier.setMinSecurityLevel(minSecurityLevel);
        verifier.setMaxProofAge(maxProofAge);

        vm.stopBroadcast();

        console.log("ZkIPFS Verifier deployed with custom params at:", address(verifier));
    }

    /// @notice Deploy for testing with minimal fees
    function deployForTesting() external {
        vm.startBroadcast();

        ZkIPFSVerifier verifier = new ZkIPFSVerifier(msg.sender, msg.sender);

        // Set testing-friendly parameters
        verifier.setVerificationFee(0); // No fees for testing
        verifier.setMinSecurityLevel(80); // Lower security for testing
        verifier.setMaxProofAge(365 days); // Long proof lifetime for testing

        vm.stopBroadcast();

        console.log("ZkIPFS Verifier deployed for testing at:", address(verifier));
    }

    /// @notice Save deployment information to a file
    function _saveDeploymentInfo(
        address verifierAddress,
        address owner,
        address feeRecipient
    ) internal {
        string memory deploymentInfo = string(abi.encodePacked(
            "{\n",
            '  "verifier": "', vm.toString(verifierAddress), '",\n',
            '  "owner": "', vm.toString(owner), '",\n',
            '  "feeRecipient": "', vm.toString(feeRecipient), '",\n',
            '  "network": "', vm.envOr("NETWORK", "localhost"), '",\n',
            '  "timestamp": "', vm.toString(block.timestamp), '"\n',
            "}"
        ));

        vm.writeFile("./deployments/latest.json", deploymentInfo);
        console.log("Deployment info saved to ./deployments/latest.json");
    }
}

