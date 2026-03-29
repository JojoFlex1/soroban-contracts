#![cfg(test)]

use crate::{CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::error::Error;

// ─────────────────────────────────────────────────────────────────────────────
// Mock RBAC Contract
// ─────────────────────────────────────────────────────────────────────────────

#[soroban_sdk::contract]
pub struct MockRbacContract;

#[soroban_sdk::contractimpl]
impl MockRbacContract {
    pub fn has_role(_env: Env, _address: Address, _role: String) -> bool {

        true // Mock: everyone is a verifier for tests
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    CarbonCreditTokenClient<'static>,
    Address, // admin
    Address, // verifier
    Address, // user
) {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, MockRbacContract);
    let token_id = env.register_contract(None, CarbonCreditToken);
    let token = CarbonCreditTokenClient::new(&env, &token_id);

    let admin    = Address::generate(&env);
    let verifier = Address::generate(&env);
    let user     = Address::generate(&env);

    token.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "Carbon Credit Token"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    env.ledger().with_mut(|l| l.timestamp = 123456789);

    (env, token, admin, verifier, user)
}

#[test]
fn test_retire_and_certificate_issuance() {
    let (_env, token, _, verifier, user) = setup();

    
    // Mint some tokens
    token.mint(&verifier, &user, &1000);
    assert_eq!(token.balance(&user), 1000);

    // Retire some tokens
    token.retire(&user, &300);
    
    assert_eq!(token.balance(&user), 700);
    assert_eq!(token.total_retired(), 300);

    // Check certificate issuance
    let certs = token.get_certificates(&user);
    assert_eq!(certs.len(), 1);
    
    let cert = certs.get(0).unwrap();
    assert_eq!(cert.id, 1);
    assert_eq!(cert.amount, 300);
    assert!(cert.timestamp > 0);
    
    assert_eq!(token.get_certificate_count(), 1);

    // Retire more
    token.retire(&user, &200);
    let certs2 = token.get_certificates(&user);
    assert_eq!(certs2.len(), 2);
    assert_eq!(certs2.get(1).unwrap().amount, 200);
    assert_eq!(token.get_certificate_count(), 2);
}

#[test]
fn test_view_certificates_empty() {
    let (env, token, _, _, _) = setup();
    let nobody = Address::generate(&env);
    let certs = token.get_certificates(&nobody);
    assert_eq!(certs.len(), 0);
}
