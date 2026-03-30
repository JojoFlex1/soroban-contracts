#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{error::Error, RbacContract, RbacContractClient};

// ── Test Helpers ───────────────────────────────────────────────────────────────

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
    assert_eq!(client.get_role(&super_admin), 0u32);
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

#[test]
fn test_get_role_verifier_is_1() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);
    client.add_verifier(&verifier);
    assert_eq!(client.get_role(&verifier), 1u32);
}

#[test]
fn test_get_role_trader_is_2() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);
    client.add_trader(&trader);
    assert_eq!(client.get_role(&trader), 2u32);
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

#[test]
fn test_super_admin_cannot_be_added_as_verifier() {
    let (_, client, super_admin) = setup();
    let result = client.try_add_verifier(&super_admin);
    assert_eq!(result, Err(Ok(Error::AddressHasDifferentRole)));
}

#[test]
fn test_multiple_verifiers() {
    let (env, client, _) = setup();

    let verifier1 = Address::generate(&env);
    let verifier2 = Address::generate(&env);
    let verifier3 = Address::generate(&env);

    client.add_verifier(&verifier1);
    client.add_verifier(&verifier2);
    client.add_verifier(&verifier3);

    assert!(client.is_verifier(&verifier1));
    assert!(client.is_verifier(&verifier2));
    assert!(client.is_verifier(&verifier3));
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
    assert_eq!(client.get_role(&trader), 255u32);
}

#[test]
fn test_remove_trader_not_assigned() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);
    let result = client.try_remove_trader(&trader);
    assert_eq!(result, Err(Ok(Error::RoleNotAssigned)));
}

#[test]
fn test_multiple_traders() {
    let (env, client, _) = setup();

    let trader1 = Address::generate(&env);
    let trader2 = Address::generate(&env);

    client.add_trader(&trader1);
    client.add_trader(&trader2);

    assert!(client.is_trader(&trader1));
    assert!(client.is_trader(&trader2));
}

// ── Role Exclusivity ───────────────────────────────────────────────────────────

#[test]
fn test_verifier_cannot_be_added_as_trader() {
    let (env, client, _) = setup();
    let addr = Address::generate(&env);

    client.add_verifier(&addr);
    let result = client.try_add_trader(&addr);
    assert_eq!(result, Err(Ok(Error::AddressHasDifferentRole)));
}

#[test]
fn test_trader_cannot_be_added_as_verifier() {
    let (env, client, _) = setup();
    let addr = Address::generate(&env);

    client.add_trader(&addr);
    let result = client.try_add_verifier(&addr);
    assert_eq!(result, Err(Ok(Error::AddressHasDifferentRole)));
}

#[test]
fn test_super_admin_cannot_be_added_as_trader() {
    let (_, client, super_admin) = setup();
    let result = client.try_add_trader(&super_admin);
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
    assert!(!client.has_role(&verifier, &String::from_str(&env, "Admin")));
}

#[test]
fn test_has_role_trader() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);

    client.add_trader(&trader);

    assert!(client.has_role(&trader, &String::from_str(&env, "Trader")));
    assert!(!client.has_role(&trader, &String::from_str(&env, "Verifier")));
}

#[test]
fn test_has_role_super_admin() {
    let (env, client, super_admin) = setup();
    assert!(client.has_role(&super_admin, &String::from_str(&env, "SuperAdmin")));
    assert!(!client.has_role(&super_admin, &String::from_str(&env, "Verifier")));
}

#[test]
fn test_has_role_unknown_address_is_false() {
    let (env, client, _) = setup();
    let nobody = Address::generate(&env);
    assert!(!client.has_role(&nobody, &String::from_str(&env, "Verifier")));
    assert!(!client.has_role(&nobody, &String::from_str(&env, "Admin")));
    assert!(!client.has_role(&nobody, &String::from_str(&env, "Trader")));
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
// ── Negative / Edge-Case Tests (Issue #49) ────────────────────────────────────

// ── Unauthorized role assignment ───────────────────────────────────────────────

#[test]
fn test_non_admin_cannot_grant_verifier_role() {
    let (env, client, _) = setup();
    let nobody = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_grant_verifier(&nobody, &target);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_non_admin_cannot_grant_trader_role() {
    let (env, client, _) = setup();
    let nobody = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_grant_trader(&nobody, &target);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_verifier_cannot_grant_roles() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);
    let target = Address::generate(&env);

    client.add_verifier(&verifier);

    let result = client.try_grant_trader(&verifier, &target);
    assert_eq!(result, Err(Ok(Error::Unauthorized)));
}

// ── SuperAdmin self-removal ────────────────────────────────────────────────────

#[test]
fn test_super_admin_cannot_revoke_own_role() {
    let (_, client, super_admin) = setup();

    let result = client.try_revoke_role(&super_admin, &super_admin);
    assert_eq!(result, Err(Ok(Error::CannotRemoveSuperAdmin)));
}

// ── Invalid role strings in has_role() ────────────────────────────────────────

#[test]
fn test_has_role_returns_false_for_empty_string() {
    let (env, client, _) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);

    assert!(!client.has_role(&verifier, &String::from_str(&env, "")));
}

#[test]
fn test_has_role_returns_false_for_garbage_role_string() {
    let (env, client, _) = setup();
    let trader = Address::generate(&env);

    client.add_trader(&trader);

    assert!(!client.has_role(&trader, &String::from_str(&env, "INVALID_ROLE")));
    assert!(!client.has_role(&trader, &String::from_str(&env, "trader")));
    assert!(!client.has_role(&trader, &String::from_str(&env, "TRADER")));
}

#[test]
fn test_has_role_is_case_sensitive() {
    let (env, client, super_admin) = setup();

    assert!(!client.has_role(&super_admin, &String::from_str(&env, "admin")));
    assert!(!client.has_role(&super_admin, &String::from_str(&env, "ADMIN")));
    assert!(!client.has_role(&super_admin, &String::from_str(&env, "superadmin")));
    assert!(!client.has_role(&super_admin, &String::from_str(&env, "super_admin")));
}

// ── transfer_admin edge cases ──────────────────────────────────────────────────

#[test]
fn test_non_super_admin_cannot_transfer_admin() {
    let (env, client, _) = setup();
    let impersonator = Address::generate(&env);
    let target = Address::generate(&env);

    let result = client.try_transfer_admin(&impersonator, &target);
    assert!(result.is_err());
}

#[test]
fn test_transfer_admin_to_existing_verifier_works() {
    let (env, client, old_admin) = setup();
    let verifier = Address::generate(&env);

    client.add_verifier(&verifier);
    assert!(client.is_verifier(&verifier));

    client.transfer_admin(&old_admin, &verifier);

    assert_eq!(client.get_super_admin(), verifier);
    assert!(client.is_super_admin(&verifier));
}

// ── Role assignment after admin transfer ──────────────────────────────────────

#[test]
fn test_old_admin_loses_super_admin_status_after_transfer() {
    let (env, client, old_admin) = setup();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&old_admin, &new_admin);

    assert!(!client.is_super_admin(&old_admin));
    assert_eq!(client.get_super_admin(), new_admin);
}

#[test]
fn test_old_admin_cannot_grant_roles_after_transfer() {
    let (env, client, old_admin) = setup();
    let new_admin = Address::generate(&env);
    let target = Address::generate(&env);

    client.transfer_admin(&old_admin, &new_admin);

    let result = client.try_add_verifier(&target);
    // old_admin no longer has super admin — any role grant it tries must fail
    let result2 = client.try_grant_verifier(&old_admin, &target);
    assert_eq!(result2, Err(Ok(Error::Unauthorized)));
}

// ── Concurrent role operations ────────────────────────────────────────────────

#[test]
fn test_assign_and_revoke_in_rapid_succession() {
    let (env, client, _) = setup();
    let addr = Address::generate(&env);

    client.add_verifier(&addr);
    assert!(client.is_verifier(&addr));

    client.remove_verifier(&addr);
    assert!(!client.is_verifier(&addr));

    // Re-assign as trader after revoking verifier — must succeed cleanly
    client.add_trader(&addr);
    assert!(client.is_trader(&addr));

    client.remove_trader(&addr);
    assert!(!client.is_trader(&addr));

    // Final state: no role
    assert_eq!(client.get_role(&addr), 255u32);
}

#[test]
fn test_role_reassignment_after_revoke() {
    let (env, client, _) = setup();
    let addr = Address::generate(&env);

    // Assign verifier, revoke, then assign verifier again — must not error
    client.add_verifier(&addr);
    client.remove_verifier(&addr);
    client.add_verifier(&addr);

    assert!(client.is_verifier(&addr));
    assert_eq!(client.get_role(&addr), 1u32);
}
