#![cfg(test)]

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, String, Symbol,
};

use crate::{CarbonCreditToken, CarbonCreditTokenClient};

// ─────────────────────────────────────────────────────────────────────────────
// Mock RBAC Contract
//
// A minimal in-process RBAC stub. The real contract lives separately; here we
// only need something that implements `has_role(address, role) -> bool` so the
// token contract's cross-contract call resolves during tests.
//
// `mock_all_auths()` makes the environment skip signature verification, so we
// don't need real keypairs — we only care about role logic.
// ─────────────────────────────────────────────────────────────────────────────

#[contract]
pub struct MockRbacContract;

/// Module-level storage key so the mock can record which address is a Verifier.
mod mock_storage {
    use soroban_sdk::{contracttype, Address};

    #[contracttype]
    pub enum MockKey {
        Verifier(Address),
    }
}

#[contractimpl]
impl MockRbacContract {
    /// Registers `address` as a Verifier in the mock store.
    pub fn grant_verifier(env: Env, address: Address) {
        env.storage()
            .instance()
            .set(&mock_storage::MockKey::Verifier(address), &true);
    }

    /// The interface method the token contract calls cross-contract.
    pub fn has_role(env: Env, address: Address, role: String) -> bool {
        // Only recognise the "Verifier" role for this mock.
        if role != String::from_str(&env, "Verifier") {
            return false;
        }
        env.storage()
            .instance()
            .get::<_, bool>(&mock_storage::MockKey::Verifier(address))
            .unwrap_or(false)
    }
}

// Generates the strongly-typed client Soroban uses to call the mock.
use soroban_sdk::contractclient;
#[contractclient(name = "MockRbacClient")]
trait MockRbacInterface {
    fn grant_verifier(env: Env, address: Address);
    fn has_role(env: Env, address: Address, role: String) -> bool;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Deploys both contracts and returns their clients plus a pre-registered
/// Verifier address and a plain user address for negative tests.
fn setup() -> (
    Env,
    CarbonCreditTokenClient<'static>,
    Address, // verifier
    Address, // non-verifier / plain user
) {
    let env = Env::default();
    env.mock_all_auths(); // skip real signature checks; focus on role logic

    // Deploy mock RBAC
    let rbac_id = env.register_contract(None, MockRbacContract);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);

    // Deploy token contract
    let token_id = env.register_contract(None, CarbonCreditToken);
    let token_client = CarbonCreditTokenClient::new(&env, &token_id);

    // Addresses
    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let user = Address::generate(&env);

    // Grant the verifier role in the RBAC mock
    rbac_client.grant_verifier(&verifier);

    // Initialize token contract — passes the RBAC contract address in
    token_client.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "CarbonCredit"),
        &String::from_str(&env, "CC"),
        &7u32,
    );

    (env, token_client, verifier, user)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// A registered Verifier can mint tokens successfully.
#[test]
fn test_verifier_can_mint() {
    let (env, token, verifier, user) = setup();

    token.mint(&verifier, &user, &1_000_i128);

    assert_eq!(token.balance(&user), 1_000_i128);
    assert_eq!(token.total_supply(), 1_000_i128);
}

/// Minting increases total supply by exactly the minted amount.
#[test]
fn test_mint_increases_total_supply() {
    let (_, token, verifier, user) = setup();

    token.mint(&verifier, &user, &500_i128);
    token.mint(&verifier, &user, &300_i128);

    assert_eq!(token.total_supply(), 800_i128);
}

/// An address that has NOT been granted the Verifier role cannot mint.
#[test]
#[should_panic(expected = "minting rejected")]
fn test_non_verifier_cannot_mint() {
    let (_, token, _, user) = setup();

    // `user` is not a Verifier — this must panic
    token.mint(&user, &user, &1_000_i128);
}

/// Passing a zero amount is rejected before the role check fires.
#[test]
#[should_panic(expected = "negative amount is not allowed")]
fn test_negative_amount_rejected() {
    let (_, token, verifier, user) = setup();

    token.mint(&verifier, &user, &-1_i128);
}

/// initialize() correctly records the RBAC contract address on-chain.
#[test]
fn test_rbac_contract_address_stored() {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, MockRbacContract);
    let token_id = env.register_contract(None, CarbonCreditToken);
    let token_client = CarbonCreditTokenClient::new(&env, &token_id);

    let admin = Address::generate(&env);

    token_client.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "CarbonCredit"),
        &String::from_str(&env, "CC"),
        &7u32,
    );

    // The view function introduced in lib.rs should return the stored address
    assert_eq!(token_client.rbac_contract(), rbac_id);
}

/// initialize() cannot be called a second time.
#[test]
#[should_panic(expected = "contract already initialized")]
fn test_double_initialize_rejected() {
    let (env, token, _, _) = setup();

    let rbac_id = env.register_contract(None, MockRbacContract);
    let admin = Address::generate(&env);

    token.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "X"),
        &String::from_str(&env, "X"),
        &7u32,
    );
}

/// Verifier auth is recorded correctly by the Soroban auth framework.
#[test]
fn test_mint_records_verifier_auth() {
    let (env, token, verifier, user) = setup();

    token.mint(&verifier, &user, &100_i128);

    // Confirm the environment captured require_auth() for the verifier
    let auths = env.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &verifier),
        "expected verifier auth to be recorded"
    );
}