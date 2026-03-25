#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{error::Error, RbacContract, RbacContractClient};

fn setup() -> (Env, RbacContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RbacContract);
    let client = RbacContractClient::new(&env, &contract_id);

    let super_admin = Address::generate(&env);
    client.initialize(&super_admin);

    (env, client, super_admin)
}

// ── Initialization ─────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_super_admin() {
    let (_, client, super_admin) = setup();
    assert_eq!(client.get_super_admin(), super_admin);
    assert!(client.is_super_admin(&super_admin));
}

#[test]
fn test_initialize_twice_fails() {
    let (env, client, _) = setup();
    let other = Address::generate(&env);
    let result = client.try_initialize(&other);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

// ── get_role ───────────────────────────────────────────────────────────────────

#[test]
fn test_get_role_returns_255_for_unknown() {
    let (env, client, _) = setup();
    let unknown = Address::generate(&env);
    assert_eq!(client.get_role(&unknown), 255u32);
}

#[test]
fn test_get_role_super_admin_is_0() {
    let (_, client, super_admin) = setup();
    assert_eq!(client.get_role(&super_admin), 0u32);
}

// ── Verifier ───────────────────────────────────────────────────────────────────

#[test]
fn test_add_and_check_verifier() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);

    assert!(client.is_verifier(&verifier));
    assert_eq!(client.get_role(&verifier), 1u32);
}

#[test]
fn test_add_verifier_twice_fails() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);
    let result = client.try_add_verifier(&verifier);
    assert_eq!(result, Err(Ok(Error::RoleAlreadyAssigned)));
}

#[test]
fn test_remove_verifier() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);
    assert!(client.is_verifier(&verifier));

    client.remove_verifier(&verifier);
    assert!(!client.is_verifier(&verifier));
    assert_eq!(client.get_role(&verifier), 255u32);
}

#[test]
fn test_remove_nonexistent_verifier_fails() {
    let (env, client, _) = setup();
    let nobody = Address::generate(&env);
    let result = client.try_remove_verifier(&nobody);
    assert_eq!(result, Err(Ok(Error::RoleNotAssigned)));
}

// ── Trader ─────────────────────────────────────────────────────────────────────

#[test]
fn test_add_and_check_trader() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);

    client.add_trader(&trader);

    assert!(client.is_trader(&trader));
    assert_eq!(client.get_role(&trader), 2u32);
}

#[test]
fn test_add_trader_twice_fails() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);

    client.add_trader(&trader);
    let result = client.try_add_trader(&trader);
    assert_eq!(result, Err(Ok(Error::RoleAlreadyAssigned)));
}

#[test]
fn test_remove_trader() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);

    client.add_trader(&trader);
    client.remove_trader(&trader);

    assert!(!client.is_trader(&trader));
}

#[test]
fn test_address_with_different_role_rejected() {
    let (env, client, _) = setup();
    let addr = Address::generate(&env);

    client.add_verifier(&addr);

    // Cannot also be a trader
    let result = client.try_add_trader(&addr);
    assert_eq!(result, Err(Ok(Error::AddressHasDifferentRole)));
}

// ── has_role (cross-contract interface) ────────────────────────────────────────

#[test]
fn test_has_role_verifier() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);

    assert!(client.has_role(&verifier, &String::from_str(&env, "Verifier")));
    assert!(!client.has_role(&verifier, &String::from_str(&env, "Trader")));
}

#[test]
fn test_has_role_unknown_address_is_false() {
    let (env, client, _) = setup();
    let nobody = Address::generate(&env);
    assert!(!client.has_role(&nobody, &String::from_str(&env, "Verifier")));
}

// ── transfer_admin ─────────────────────────────────────────────────────────────

#[test]
fn test_transfer_admin() {
    let (env, client, old_admin) = setup();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&old_admin, &new_admin);

    assert_eq!(client.get_super_admin(), new_admin);
    assert!(client.is_super_admin(&new_admin));
    assert!(!client.is_super_admin(&old_admin));
}

#[test]
fn test_transfer_admin_unauthorized() {
    let (env, client, _) = setup();
    let impostor = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let result = client.try_transfer_admin(&impostor, &new_admin);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

// ── revoke_role ────────────────────────────────────────────────────────────────

#[test]
fn test_revoke_role_via_legacy_interface() {
    let (env, client, super_admin) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);
    client.revoke_role(&super_admin, &verifier);

    assert!(!client.is_verifier(&verifier));
}

#[test]
fn test_revoke_role_cannot_revoke_admin() {
    let (_, client, super_admin) = setup();
    let result = client.try_revoke_role(&super_admin, &super_admin);
    assert_eq!(result, Err(Ok(Error::CannotRevokeAdmin)));
}