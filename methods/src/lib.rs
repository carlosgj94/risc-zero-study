// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Generated crate containing the image ID and ELF binary of the build guest.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256};
    use alloy_sol_types::SolValue;
    use risc0_steel::{
        config::ETH_SEPOLIA_CHAIN_SPEC,
        ethereum::{EthEvmEnv, EthEvmInput},
    };
    use risc0_zkvm::{default_executor, guest::env, ExecutorEnv};

    #[test]
    fn proves_even_number() {
        let even_number = U256::from(1304);

        let env = ExecutorEnv::builder()
            .write_slice(&even_number.abi_encode())
            .build()
            .unwrap();

        // NOTE: Use the executor to run tests without proving.
        let session_info = default_executor().execute(env, super::IS_EVEN_ELF).unwrap();

        let x = U256::abi_decode(&session_info.journal.bytes, true).unwrap();
        assert_eq!(x, even_number);
    }

    #[test]
    fn proves_voting() {
        let evm_input: EthEvmInput;
        let example_signature = [27, 0x8f, 0x7f, 0x3e];
        let voter = Address::parse_checksummed("0xB0b", None).unwrap();
        let dao = Address::parse_checksummed("0xDA0", None).unwrap();
        let proposal_id: U256 = U256::from(0x0);
        let direction = [1];
        let balance: U256 = U256::from(0x0);
        let token_contract =
            Address::parse_checksummed("0x67a53a2b9984AF64A2e27b1582bC72406a2317c3", None).unwrap();

        let env = ExecutorEnv::builder()
            .write_slice(&evm_input)
            .write_slice(&example_signature.abi_encode())
            .write_slice(&voter.abi_encode())
            .write_slice(&dao.abi_encode())
            .write_slice(&proposal_id.abi_encode())
            .write_slice(&direction)
            .write_slice(&balance.abi_encode())
            .write_slice(&token_contract.abi_encode())
            .build()
            .unwrap();

        default_executor().execute(env, super::VOTE_ELF).unwrap();
    }
}
