// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Verifier interface for zk-SNARK proof verification
interface IPlonkVerifier {
    function verifyProof(bytes memory proof, uint256[] memory pubSignals) external view returns (bool);
}

contract ComputeContract {
    address public nodeOperator;
    IPlonkVerifier public verifier;

    event ComputeRequested(address indexed client, uint256 amount);
    event ComputeCompleted(address indexed dataProvider, uint256 nodeOperatorShare, uint256 dataProviderShare);
    event ProofVerified(bool verified);

    constructor(address _verifier) {
        nodeOperator = msg.sender;
        verifier = IPlonkVerifier(_verifier);
    }

    // Function for client to request computation and send payment
    function requestCompute() external payable {
        require(msg.value > 0, "You must send some ETH for computation");
        emit ComputeRequested(msg.sender, msg.value);
    }

    // Function for node operator to complete computation and distribute funds with zk-SNARK proof
    function completeCompute(address dataProvider, bytes memory proof, uint256[] memory pubSignals) external {
        require(msg.sender == nodeOperator, "Only the node operator can call this function");

        // Verify the zk-SNARK proof
        bool verified = verifier.verifyProof(proof, pubSignals);
        emit ProofVerified(verified);
        require(verified, "Invalid proof");

        uint256 totalAmount = address(this).balance;
        uint256 nodeOperatorShare = (totalAmount * 70) / 100;
        uint256 dataProviderShare = totalAmount - nodeOperatorShare; // Remaining 30%

        payable(nodeOperator).transfer(nodeOperatorShare);
        payable(dataProvider).transfer(dataProviderShare);

        emit ComputeCompleted(dataProvider, nodeOperatorShare, dataProviderShare);
    }
}
