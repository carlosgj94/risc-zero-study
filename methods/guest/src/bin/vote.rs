use risc0_zkvm::guest::env;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};
use risc0_steel::{config::ETH_SEPOLIA_CHAIN_SPEC, ethereum::EthEvmInput, Contract, SolCommitment};

use k256::{
    ecdsa::{RecoveryId, Signature, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
    PublicKey,
};
use tiny_keccak::{Hasher, Keccak};

sol! {
    /// ERC-20 balance function signature.
    interface IERC20 {
        function balanceOf(address account) external view returns (uint);
    }
}

sol! {
    interface Steel {
        struct Journal {
            SolCommitment commitment;
            address tokenAddress;
            address voter;
            uint256 balance;
            uint8 direction;
        }
    }
}

const PREFIX: &str = "\x19Ethereum Signed Message:\n32";

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut digest = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut digest);
    digest
}

/// Converts an Ethereum-convention recovery ID to the k256 RecoveryId type.
fn into_recovery_id(v: u8) -> Option<RecoveryId> {
    match v {
        0 => Some(0),
        1 => Some(1),
        27 => Some(0),
        28 => Some(1),
        v if v >= 35 => Some((v - 1) % 2),
        _ => None,
    }
    .and_then(RecoveryId::from_byte)
}

/// Signer address recovery from the (v, r, s) signature components.
///
/// This methods exists to replicate the behavior of `ecrecover` within the EVM.
/// It can only be considered a signature validation is digest is verified to be
/// the hash of a known message.
fn ecrecover(v: u8, rs: [u8; 64], digest: [u8; 32]) -> [u8; 20] {
    let recovery_id = into_recovery_id(v).expect("value for v is invalid");
    let signature = Signature::from_slice(&rs[..]).expect("signature encoding is invalid");
    let recovered_pk: PublicKey =
        VerifyingKey::recover_from_prehash(&digest[..], &signature, recovery_id)
            .expect("signature is invalid")
            .into();

    // Calculate the Ethereum address from the k256 public key.
    let encoded_pk = recovered_pk.to_encoded_point(/* compress = */ false);
    keccak256(&encoded_pk.as_bytes()[1..])[12..]
        .try_into()
        .unwrap()
}

fn hash_vote(
    chain_id: u64,
    dao: Address,
    proposal_id: U256,
    direction: u8,
    balance: U256,
) -> [u8; 32] {
    let message_hash = keccak256(
        &[
            chain_id.to_be_bytes().to_vec(),
            dao.to_vec(),
            proposal_id.to_be_bytes_vec(),
            [direction].to_vec(),
            balance.to_be_bytes_vec(),
        ]
        .concat(),
    );
    let prefixed_message = [PREFIX.as_bytes(), &message_hash].concat();
    keccak256(&prefixed_message)
}

fn main() {
    // Read the input from the guest environment.
    let input: EthEvmInput = env::read();
    let signature: Signature = env::read();
    let account: Address = env::read();
    let dao: Address = env::read();
    let proposal_id: U256 = env::read();
    let direction: u8 = env::read();
    let balance: U256 = env::read();
    let token_contract: Address = env::read();

    // Converts the input into a `EvmEnv` for execution. The `with_chain_spec` method is used
    // to specify the chain configuration. It checks that the state matches the state root in the
    // header provided in the input.
    let env = input.into_env().with_chain_spec(&ETH_SEPOLIA_CHAIN_SPEC);

    let digest = hash_vote(
        ETH_SEPOLIA_CHAIN_SPEC.chain_id(),
        dao,
        proposal_id,
        direction,
        balance,
    );
    let v = signature.to_bytes()[64];
    let rs = signature.to_bytes()[0..64].try_into().unwrap();
    let signature_address = ecrecover(v, rs, digest);

    // Execute the view call; it returns the result in the type generated by the `sol!` macro.
    let call = IERC20::balanceOfCall { account };
    let balance_of_call_result = Contract::new(token_contract, &env)
        .call_builder(&call)
        .call();

    // General settings constraints
    assert!(direction == 0 || direction == 1);

    assert!(balance > U256::from(0));
    assert!(account == signature_address);
    assert!(balance == balance_of_call_result._0);

    // Commit the block hash and number used when deriving `view_call_env` to the journal.
    // The portion of the receipt that contains the public outputs of a zkVM application.
    let journal = Steel::Journal {
        commitment: env.block_commitment(),
        tokenAddress: token_contract,
        voter: account,
        balance,
        direction,
    };

    env::commit_slice(&journal.abi_encode());
}
