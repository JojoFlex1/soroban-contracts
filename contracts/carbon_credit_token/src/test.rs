#![cfg(test)]

use crate::{error::Error, CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, String,
};

fn create_token<'a>(e: &Env, admin: &Address) -> CarbonCreditTokenClient<'a> {
    let contract_id = e.register_contract(None, CarbonCreditToken);
    let client = CarbonCreditTokenClient::new(e, &contract_id);

    client.initialize(
        admin,
        &String::from_str(e, "Carbon Credit Token"),
        &String::from_str(e, "CCT"),
        &0u32,
    );

    client
}

// ============ INITIALIZATION TESTS ============

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.name(), String::from_str(&env, "Carbon Credit Token"));
    assert_eq!(token.symbol(), String::from_str(&env, "CCT"));
    assert_eq!(token.decimals(), 0u32);
    assert_eq!(token.total_supply(), 0i128);
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

// ============ MINT TESTS ============

#[test]
fn test_mint() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.total_supply(), 1000);
}

#[test]
fn test_mint_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.mint(&user2, &500);

    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 500);
    assert_eq!(token.total_supply(), 1500);
}

#[test]
fn test_mint_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &0);

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
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

// ============ TRANSFER TESTS ============

#[test]
fn test_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.transfer(&user1, &user2, &400);

    assert_eq!(token.balance(&user1), 600);
    assert_eq!(token.balance(&user2), 400);
    assert_eq!(token.total_supply(), 1000);
}

#[test]
fn test_transfer_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.transfer(&user1, &user2, &0);

    assert_eq!(token.balance(&user1), 1000);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
fn test_transfer_full_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
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

// ============ RETIRE TESTS ============

#[test]
fn test_retire() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.retire(&user1, &300);

    assert_eq!(token.balance(&user1), 700);
    assert_eq!(token.total_supply(), 700);
    assert_eq!(token.total_retired(), 300);
}

#[test]
fn test_retire_multiple_times() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.retire(&user1, &200);
    token.retire(&user1, &300);

    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.total_supply(), 500);
    assert_eq!(token.total_retired(), 500);
}

#[test]
fn test_retire_full_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.retire(&user1, &1000);

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.total_supply(), 0);
    assert_eq!(token.total_retired(), 1000);
}

#[test]
fn test_retire_by_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.mint(&user2, &500);

    token.retire(&user1, &300);
    token.retire(&user2, &200);

    assert_eq!(token.balance(&user1), 700);
    assert_eq!(token.balance(&user2), 300);
    assert_eq!(token.total_supply(), 1000);
    assert_eq!(token.total_retired(), 500);
}

#[test]
fn test_retire_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    let result = token.try_retire(&user1, &0);
    assert_eq!(result, Err(Ok(Error::ZeroRetirementAmount)));
}

#[test]
fn test_retire_insufficient_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &100);

    let result = token.try_retire(&user1, &500);
    assert_eq!(result, Err(Ok(Error::InsufficientBalance)));
}

// ============ BURN TESTS ============

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.burn(&user1, &300);

    assert_eq!(token.balance(&user1), 700);
    assert_eq!(token.total_supply(), 700);
    // burn does NOT increment total_retired
    assert_eq!(token.total_retired(), 0);
}

#[test]
fn test_burn_vs_retire_difference() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    // Burn reduces supply but NOT total_retired
    token.burn(&user1, &200);
    assert_eq!(token.total_supply(), 800);
    assert_eq!(token.total_retired(), 0);

    // Retire reduces supply AND increments total_retired
    token.retire(&user1, &200);
    assert_eq!(token.total_supply(), 600);
    assert_eq!(token.total_retired(), 200);
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

// ============ ALLOWANCE TESTS ============

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    let expiration = env.ledger().sequence() + 1000;
    token.approve(&user1, &spender, &500, &expiration);

    assert_eq!(token.allowance(&user1, &spender), 500);

    token.transfer_from(&spender, &user1, &user2, &200);

    assert_eq!(token.balance(&user1), 800);
    assert_eq!(token.balance(&user2), 200);
    assert_eq!(token.allowance(&user1, &spender), 300);
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
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    let expiration = env.ledger().sequence() + 1000;
    token.approve(&user1, &spender, &500, &expiration);

    token.burn_from(&spender, &user1, &200);

    assert_eq!(token.balance(&user1), 800);
    assert_eq!(token.total_supply(), 800);
    assert_eq!(token.allowance(&user1, &spender), 300);
}

#[test]
fn test_approve_updates_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    let expiration = env.ledger().sequence() + 1000;

    token.approve(&user1, &spender, &500, &expiration);
    assert_eq!(token.allowance(&user1, &spender), 500);

    // Update allowance
    token.approve(&user1, &spender, &300, &expiration);
    assert_eq!(token.allowance(&user1, &spender), 300);
}

#[test]
fn test_approve_expired_ledger_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    // expiration_ledger is in the past (sequence is 0 by default, but 0 < 0 is false,
    // so we need sequence > expiration; bump ledger sequence first)
    let past_ledger: u32 = 0;
    // amount > 0 with expiration_ledger < current sequence triggers the error.
    // Default sequence is 0, so we need to advance it.
    env.ledger().set_sequence_number(10);

    let result = token.try_approve(&user1, &spender, &100, &past_ledger);
    assert_eq!(result, Err(Ok(Error::InvalidExpirationLedger)));
}

// ============ EDGE CASES ============

#[test]
fn test_balance_uninitialized_address() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unknown = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.balance(&unknown), 0);
}

#[test]
fn test_allowance_uninitialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let spender = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.allowance(&user1, &spender), 0);
}

// ============ EVENT VERIFICATION TESTS ============

#[test]
fn test_mint_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);

    let events = env.events().all();
    assert!(events.len() >= 1);
}

#[test]
fn test_transfer_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.transfer(&user1, &user2, &500);

    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
fn test_retirement_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let token = create_token(&env, &admin);

    token.mint(&user1, &1000);
    token.retire(&user1, &300);

    let events = env.events().all();
    // retire emits both RetirementEvent and BurnEvent
    assert!(events.len() >= 3);
}

// ============ METADATA TESTS ============

#[test]
fn test_metadata_values() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.name(), String::from_str(&env, "Carbon Credit Token"));
    assert_eq!(token.symbol(), String::from_str(&env, "CCT"));
    assert_eq!(token.decimals(), 0u32);
}

// ============ SUPPLY TRACKING TESTS ============

#[test]
fn test_total_supply_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.total_supply(), 0);

    token.mint(&user1, &1000);
    assert_eq!(token.total_supply(), 1000);

    token.mint(&user2, &500);
    assert_eq!(token.total_supply(), 1500);

    token.burn(&user1, &200);
    assert_eq!(token.total_supply(), 1300);

    token.retire(&user2, &100);
    assert_eq!(token.total_supply(), 1200);
}

#[test]
fn test_total_retired_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin);

    assert_eq!(token.total_retired(), 0);

    token.mint(&user1, &1000);
    token.mint(&user2, &500);

    token.retire(&user1, &200);
    assert_eq!(token.total_retired(), 200);

    token.retire(&user2, &150);
    assert_eq!(token.total_retired(), 350);

    // burn does NOT increment total_retired
    token.burn(&user1, &100);
    assert_eq!(token.total_retired(), 350);
}

// ============ COMPLEX SCENARIO TESTS ============

#[test]
fn test_full_token_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let donor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token = create_token(&env, &admin);

    // 1. Admin mints tokens for a verified donation
    token.mint(&donor, &1000);
    assert_eq!(token.balance(&donor), 1000);
    assert_eq!(token.total_supply(), 1000);

    // 2. Donor transfers some tokens to a recipient
    token.transfer(&donor, &recipient, &300);
    assert_eq!(token.balance(&donor), 700);
    assert_eq!(token.balance(&recipient), 300);

    // 3. Recipient retires tokens to claim carbon offset
    token.retire(&recipient, &100);
    assert_eq!(token.balance(&recipient), 200);
    assert_eq!(token.total_supply(), 900);
    assert_eq!(token.total_retired(), 100);

    // 4. Donor also retires some tokens
    token.retire(&donor, &200);
    assert_eq!(token.balance(&donor), 500);
    assert_eq!(token.total_supply(), 700);
    assert_eq!(token.total_retired(), 300);

    // Verify total CO2 offset: 300 kg
    assert_eq!(token.total_retired(), 300);
}
