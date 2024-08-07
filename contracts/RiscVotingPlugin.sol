// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {Steel} from "risc0/steel/Steel.sol";
import {ImageID} from "./ImageID.sol"; // auto-generated contract after running `cargo build`.

/// @title A starter application using RISC Zero.
/// @notice This basic application holds a number, guaranteed to be even.
/// @dev This contract demonstrates one pattern for offloading the computation of an expensive
///      or difficult to implement function to a RISC Zero guest running on Bonsai.
contract RiscVotingPlugin {
    /// @notice RISC Zero verifier contract address.
    IRiscZeroVerifier public immutable verifier;
    /// @notice Image ID of the only zkVM binary to accept verification from.
    ///         The image ID is similar to the address of a smart contract.
    ///         It uniquely represents the logic of that guest program,
    ///         ensuring that only proofs generated from a pre-defined guest program
    ///         (in this case, checking if a number is even) are considered valid.
    bytes32 public constant imageId = ImageID.VOTE_ID;

    /// @notice A number that is guaranteed, by the RISC Zero zkVM, to be even.
    ///         It can be set by calling the `set` function.
    uint256 public proposalId;
    address public dao;
    address public token;

    struct Journal {
        Steel.Commitment commitment;
        address token;
        address dao;
        address voter;
        uint256 proposalId;
        uint8 direction;
        uint256 votingPower;
    }

    /// @notice Initialize the contract, binding it to a specified RISC Zero verifier.
    constructor(IRiscZeroVerifier _verifier, address _dao, address _token) {
        verifier = _verifier;
        proposalId = 0;
        dao = _dao;
        token = _token;
    }

    /// @notice Set the even number stored on the contract. Requires a RISC Zero proof that the number is even.
    function vote(bytes calldata journalData, bytes calldata seal) public {
        Journal memory journal = abi.decode(journalData, (Journal));

        // Ensure the data provided in the Journal is actually valid
        require(journal.token == token, "Invalid token");
        require(journal.proposalId == proposalId, "Invalid proposalId");
        require(journal.dao == dao, "Invalid DAO");

        // Validating the jounral data
        require(
            Steel.validateCommitment(journal.commitment),
            "Invalid commitment"
        );

        // Verify the proof
        bytes32 journalHash = sha256(journalData);
        verifier.verify(seal, imageId, journalHash);

        // Execute whatever
        proposalId = journal.direction;
    }

    /// @notice Returns the number stored.
    function get() public view returns (uint256) {
        return proposalId;
    }
}
