// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {MockERC20} from "forge-std/mocks/MockERC20.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {RiscVotingPlugin} from "../contracts/RiscVotingPlugin.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

contract Token_ERC20 is MockERC20 {
    constructor(string memory name, string memory symbol, uint8 decimals) {
        initialize(name, symbol, decimals);
    }

    function mint(address to, uint256 value) public virtual {
        _mint(to, value);
    }

    function burn(address from, uint256 value) public virtual {
        _burn(from, value);
    }
}

contract RiscVotingPluginTest is RiscZeroCheats, Test {
    RiscVotingPlugin public votingPlugin;
    Token_ERC20 public token;
    address public alice;
    uint256 public alicePk;
    address public dao;

    function setUp() public {
        (alice, alicePk) = makeAddrAndKey("alice");
        token = new Token_ERC20("R0", "R0", 18);
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        votingPlugin = new RiscVotingPlugin(
            verifier,
            address(0),
            address(token)
        );
        token.mint(alice, 1 ether);
    }

    function test_Vote() public {
        bytes32 votingDataHash = keccak256(
            abi.encode(block.chainid, dao, 0, 1, 1 ether)
        );

        bytes memory prefixedMessage = abi.encodePacked(
            "\x19Ethereum Signed Message:\n32",
            votingDataHash
        );

        bytes32 voteHash = keccak256(prefixedMessage);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(alicePk, voteHash);
        bytes memory signature = abi.encodePacked(r, s, v);

        // TODO: Check on the inputs of the prove
        // Probably I need sending the rpc params?
        (bytes memory journal, bytes memory seal) = prove(
            Elf.VOTE_PATH,
            abi.encode(signature, alice, dao, 0, 1, 1 ether, token)
        );

        votingPlugin.vote(journal, seal);
        assertEq(votingPlugin.get(), 1);
    }
}
