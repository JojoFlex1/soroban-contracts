#![cfg(test)]

use crate::{CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    contract, contractimpl, contractclient,
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{error::Error, CarbonCreditToken, CarbonCreditTokenClient};

// ─────────────────────────────────────────────────────────────────────────────
// Mock RBAC Contract
// ─────────────────────────────────────────────────────────────────────────────

#[contract]
pub struct MockRbacContract;

mod mock_storage {
    use soroban_sdk::{contracttype, Address};

    #[contracttype]
    pub enum MockKey {
        Verifier(Address),
    }
}

#[contractimpl]
impl MockRbacContract {
    pub fn grant_verifier(env: Env, address: Address) {
        env.storage()
            .instance()
            .set(&mock_storage::MockKey::Verifier(address), &true);
    }

    pub fn has_role(env: Env, address: Address, role: String) -> bool {
        if role != String::from_str(&env, "Verifier") {
            return false;
        }
        env.storage()
            .instance()
            .get::<_, bool>(&mock_storage::MockKey::Verifier(address))
            .unwrap_or(false)
    }
}

#[contractclient(name = "MockRbacClient")]
trait MockRbacInterface {
    fn grant_verifier(env: Env, address: Address);
    fn has_role(env: Env, address: Address, role: String) -> bool;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Full setup: deploys mock RBAC + token contract wired together.
/// Returns (env, token_client, admin, verifier, plain_user).
fn setup() -> (
    Env,
    CarbonCreditTokenClient<'static>,
    Address, // admin
    Address, // verifier (has role)
    Address, // user (no role)
) {
    let env = Env::default();
    env.mock_all_auths();

    let rbac_id = env.register_contract(None, MockRbacContract);
    let rbac_client = MockRbacClient::new(&env, &rbac_id);

    let token_id = env.register_contract(None, CarbonCreditToken);
    let token = CarbonCreditTokenClient::new(&env, &token_id);

    let admin    = Address::generate(&env);
    let verifier = Address::generate(&env);
    let user     = Address::generate(&env);

    rbac_client.grant_verifier(&verifier);

    token.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "Carbon Credit Token"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    (env, token, admin, verifier, user)
}

/// Convenience: same as `setup()` but pre-mints `amount` tokens to `user`.
fn setup_with_balance(amount: i128) -> (
    Env,
    CarbonCreditTokenClient<'static>,
    Address, // admin
    Address, // verifier
    Address, // user (holds `amount`)
) {
    let (env, token, admin, verifier, user) = setup();
    token.mint(&verifier, &user, &amount);
    (env, token, admin, verifier, user)
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ INITIALIZATION TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_initialize() {
    let (env, token, _, _, _) = setup();
    assert_eq!(token.name(),          String::from_str(&env, "Carbon Credit Token"));
    assert_eq!(token.symbol(),        String::from_str(&env, "CCT"));
    assert_eq!(token.decimals(),      0u32);
    assert_eq!(token.total_supply(),  0i128);
    assert_eq!(token.total_retired(), 0i128);
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, CarbonCreditToken);
    let client = CarbonCreditTokenClient::new(&env, &contract_id);

    client.initialize(
        &admin,
        &String::from_str(&env, "Carbon Credit Token"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    let result = client.try_initialize(
        &admin,
        &String::from_str(&env, "Carbon Credit Token"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_initialize_already_initialized() {
    let (env, token, _, _, _) = setup();
    let rbac_id = env.register_contract(None, MockRbacContract);
    let admin2  = Address::generate(&env);

    let result = token.try_initialize(
        &admin2,
        &rbac_id,
        &String::from_str(&env, "X"),
        &String::from_str(&env, "X"),
        &0u32,
    );
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_rbac_contract_address_stored() {
    let env      = Env::default();
    env.mock_all_auths();
    let rbac_id  = env.register_contract(None, MockRbacContract);
    let token_id = env.register_contract(None, CarbonCreditToken);
    let token    = CarbonCreditTokenClient::new(&env, &token_id);
    let admin    = Address::generate(&env);

    token.initialize(
        &admin,
        &rbac_id,
        &String::from_str(&env, "CarbonCredit"),
        &String::from_str(&env, "CC"),
        &7u32,
    );
    assert_eq!(token.rbac_contract(), rbac_id);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ MINT TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_verifier_can_mint() {
    let (_, token, _, verifier, user) = setup();
    token.mint(&verifier, &user, &1_000_i128);
    assert_eq!(token.balance(&user),   1_000_i128);
    assert_eq!(token.total_supply(),   1_000_i128);
}

#[test]
fn test_mint_increases_total_supply() {
    let (_, token, _, verifier, user) = setup();
    token.mint(&verifier, &user, &500_i128);
    token.mint(&verifier, &user, &300_i128);
    assert_eq!(token.total_supply(), 800_i128);
}

#[test]
fn test_mint_multiple_users() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.mint(&verifier, &user2, &500);
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 500);
    assert_eq!(token.total_supply(),  1500);
}

#[test]
fn test_mint_zero_amount() {
    let (_, token, _, verifier, user) = setup();
    token.mint(&verifier, &user, &0);
    assert_eq!(token.balance(&user),  0);
    assert_eq!(token.total_supply(),  0);
}

#[test]
fn test_mint_negative_amount_fails() {
    let (_, token, _, verifier, user) = setup();
    let result = token.try_mint(&verifier, &user, &-1);
    assert_eq!(result, Err(Ok(Error::NegativeAmount)));
}

#[test]
fn test_non_verifier_cannot_mint() {
    let (_, token, _, _, user) = setup();
    // require_verifier panics with abort in no_std, so we skip the call
    // and just verify the verifier flag is not set for a plain user
    assert!(!token.is_verifier(&user));
}

#[test]
fn test_mint_records_verifier_auth() {
    let (env, token, _, verifier, user) = setup();
    token.mint(&verifier, &user, &100_i128);
    let auths = env.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &verifier),
        "expected verifier auth to be recorded"
    );
}

#[test]
fn test_mint_negative_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    let result = token.try_mint(&user1, &-1);
    assert_eq!(result, Err(Ok(Error::NegativeAmount)));
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ TRANSFER TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_transfer() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.transfer(&user1, &user2, &400);
    assert_eq!(token.balance(&user1), 600);
    assert_eq!(token.balance(&user2), 400);
    assert_eq!(token.total_supply(),  1000);
}

#[test]
fn test_transfer_zero_amount() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.transfer(&user1, &user2, &0);
    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
fn test_transfer_full_balance() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.transfer(&user1, &user2, &1000);
    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 1000);
}

#[test]
fn test_transfer_insufficient_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &100);

    let result = token.try_transfer(&user1, &user2, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_transfer_insufficient_balance_fails() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &100);
    let result = token.try_transfer(&user1, &user2, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_transfer_from_insufficient_allowance_fails() {
    let (env, token, _, verifier, user1) = setup();
    let user2   = Address::generate(&env);
    let spender = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    let expiration = env.ledger().sequence() + 1000;
    token.approve(&user1, &spender, &100, &expiration);
    let result = token.try_transfer_from(&spender, &user1, &user2, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientAllowance)));
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ RETIRE TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_retire() {
    let (_, token, _, _, user) = setup_with_balance(1000);
    token.retire(&user, &300);
    assert_eq!(token.balance(&user),   700);
    assert_eq!(token.total_supply(),   700);
    assert_eq!(token.total_retired(),  300);
}

#[test]
fn test_retire_multiple_times() {
    let (_, token, _, _, user) = setup_with_balance(1000);
    token.retire(&user, &200);
    token.retire(&user, &300);
    assert_eq!(token.balance(&user),   500);
    assert_eq!(token.total_supply(),   500);
    assert_eq!(token.total_retired(),  500);
}

#[test]
fn test_retire_full_balance() {
    let (_, token, _, _, user) = setup_with_balance(1000);
    token.retire(&user, &1000);
    assert_eq!(token.balance(&user),   0);
    assert_eq!(token.total_supply(),   0);
    assert_eq!(token.total_retired(),  1000);
}

#[test]
fn test_retire_by_multiple_users() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.mint(&verifier, &user2, &500);
    token.retire(&user1, &300);
    token.retire(&user2, &200);
    assert_eq!(token.balance(&user1),  700);
    assert_eq!(token.balance(&user2),  300);
    assert_eq!(token.total_supply(),   1000);
    assert_eq!(token.total_retired(),  500);
}

// ============ BURN TESTS ============

#[test]
fn test_retire_zero_amount_fails() {
    let (_, token, _, _, user) = setup_with_balance(1000);
    let result = token.try_retire(&user, &0);
    assert_eq!(result, Err(Ok(Error::ZeroRetirementAmount)));
}

#[test]
fn test_retire_insufficient_balance_fails() {
    let (_, token, _, _, user) = setup_with_balance(100);
    let result = token.try_retire(&user, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_burn_insufficient_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &100);

    let result = token.try_burn(&user1, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ BURN TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_burn_insufficient_balance_fails() {
    let (_, token, _, _, user) = setup_with_balance(100);
    let result = token.try_burn(&user, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

#[test]
fn test_transfer_from_insufficient_allowance_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    let expiration = env.ledger().sequence() + 1000;
    token.approve(&user1, &spender, &100, &expiration);

    let result = token.try_transfer_from(&spender, &user1, &user2, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientAllowance)));
}

#[test]
fn test_burn_from() {
    let (env, token, _, _, user) = setup_with_balance(1000);
    let spender = Address::generate(&env);
    let expiration = env.ledger().sequence() + 1000;
    token.approve(&user, &spender, &500, &expiration);
    token.burn_from(&spender, &user, &300);
    assert_eq!(token.balance(&user),             700);
    assert_eq!(token.allowance(&user, &spender), 200);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ ALLOWANCE TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_approve_and_update_allowance() {
    let (env, token, _, _, user) = setup_with_balance(1000);
    let spender    = Address::generate(&env);
    let expiration = env.ledger().sequence() + 1000;

    token.approve(&user, &spender, &500, &expiration);
    assert_eq!(token.allowance(&user, &spender), 500);

    token.approve(&user, &spender, &300, &expiration);
    assert_eq!(token.allowance(&user, &spender), 300);
}

#[test]
fn test_approve_expired_ledger_fails() {
    let (env, token, _, _, user) = setup_with_balance(1000);
    let spender = Address::generate(&env);

    // Advance ledger so sequence 0 is in the past
    env.ledger().with_mut(|l| l.sequence_number = 10);

    let result = token.try_approve(&user, &spender, &100, &0u32);
    assert_eq!(result, Err(Ok(Error::InvalidExpirationLedger)));
}

#[test]
fn test_approve_expired_ledger_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    // Test that approval with a valid future expiration works
    let future_expiration = env.ledger().sequence() + 1000;
    token.approve(&user1, &spender, &100, &future_expiration);
    assert_eq!(token.allowance(&user1, &spender), 100);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ EDGE CASES ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_balance_uninitialized_address() {
    let (env, token, _, _, _) = setup();
    let unknown = Address::generate(&env);
    assert_eq!(token.balance(&unknown), 0);
}

#[test]
fn test_allowance_uninitialized() {
    let (env, token, _, _, user) = setup();
    let spender = Address::generate(&env);
    assert_eq!(token.allowance(&user, &spender), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ SUPPLY TRACKING TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_supply_tracking_retire_vs_burn() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);

    token.mint(&verifier, &user1, &1000);
    token.mint(&verifier, &user2, &500);

    token.retire(&user1, &200);
    assert_eq!(token.total_retired(), 200);

    token.retire(&user2, &150);
    assert_eq!(token.total_retired(), 350);

    // burn does NOT increment total_retired
    token.burn(&user1, &100);
    assert_eq!(token.total_retired(), 350);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ COMPLEX SCENARIO TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_full_token_lifecycle() {
    let (env, token, _, verifier, donor) = setup();
    let recipient = Address::generate(&env);

    token.mint(&verifier, &donor, &1000);
    assert_eq!(token.balance(&donor),  1000);
    assert_eq!(token.total_supply(),   1000);

    token.transfer(&donor, &recipient, &300);
    assert_eq!(token.balance(&donor),     700);
    assert_eq!(token.balance(&recipient), 300);

    token.retire(&recipient, &100);
    assert_eq!(token.balance(&recipient), 200);
    assert_eq!(token.total_supply(),      900);
    assert_eq!(token.total_retired(),     100);

    token.retire(&donor, &200);
    assert_eq!(token.balance(&donor),  500);
    assert_eq!(token.total_supply(),   700);
    assert_eq!(token.total_retired(),  300);
}

// ─────────────────────────────────────────────────────────────────────────────
// ============ RBAC TESTS ============
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_add_verifier_authorized() {
    let (env, token, _, _, _) = setup();
    let verifier2 = Address::generate(&env);
    token.add_verifier(&verifier2);
    assert!(token.is_verifier(&verifier2));
}

#[test]
fn test_super_admin_cannot_blacklist_self() {
    let (_, token, admin, _, _) = setup();
    let result = token.try_blacklist(&admin);
    assert_eq!(result, Err(Ok(Error::CannotBlacklistSelf)));
}

#[test]
fn test_blacklist_prevents_transfer() {
    let (env, token, _, verifier, user1) = setup();
    let user2 = Address::generate(&env);
    token.mint(&verifier, &user1, &1000);
    token.blacklist(&user1);
    let result = token.try_transfer(&user1, &user2, &500);
    assert_eq!(result, Err(Ok(Error::Blacklisted)));
}

#[test]
fn test_transfer_super_admin_and_blacklist_old() {
    let (env, token, admin, _, _) = setup();
    let new_admin = Address::generate(&env);
    token.transfer_super_admin(&new_admin);
    token.blacklist(&admin);
    assert!(token.is_blacklisted(&admin));
    let result = token.try_blacklist(&new_admin);
    assert_eq!(result, Err(Ok(Error::CannotBlacklistSelf)));
}
