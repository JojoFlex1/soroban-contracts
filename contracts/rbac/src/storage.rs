use soroban_sdk::{contracttype, Address, Env};

// Storage TTL bump values used to keep state around during upgrades.
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days

// ── Role Types ───────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum RoleType {
    SuperAdmin,
    Verifier,
    Trader,
}

// ── Storage Keys ─────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Initialized,
    SuperAdmin,
    Role(Address),
}

// ── Initialization ────────────────────────────────────────────────────────────
pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(e: &Env) {
    e.storage()
        .instance()
        .set(&DataKey::Initialized, &true);
}

// ── SuperAdmin ────────────────────────────────────────────────────────────────
pub fn read_super_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::SuperAdmin)
        .unwrap()
}

pub fn write_super_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, admin);
}

// ── Role Helpers ──────────────────────────────────────────────────────────────
pub fn read_role(e: &Env, address: &Address) -> Option<RoleType> {
    e.storage()
        .persistent()
        .get(&DataKey::Role(address.clone()))
}

pub fn write_role(e: &Env, address: &Address, role: RoleType) {
    e.storage()
        .persistent()
        .set(&DataKey::Role(address.clone()), &role);
}

pub fn remove_role(e: &Env, address: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Role(address.clone()));
}

// ── Role Write Helpers ───────────────────────────────────────────────────────
pub fn grant_super_admin(e: &Env, address: &Address) {
    write_role(e, address, RoleType::SuperAdmin);
}

pub fn revoke_super_admin(e: &Env, address: &Address) {
    remove_role(e, address);
}

pub fn grant_verifier(e: &Env, address: &Address) {
    write_role(e, address, RoleType::Verifier);
}

pub fn revoke_verifier(e: &Env, address: &Address) {
    remove_role(e, address);
}

pub fn grant_trader(e: &Env, address: &Address) {
    write_role(e, address, RoleType::Trader);
}

pub fn revoke_trader(e: &Env, address: &Address) {
    remove_role(e, address);
}

// ── Role Checks ───────────────────────────────────────────────────────────────
pub fn is_super_admin(e: &Env, address: &Address) -> bool {
    matches!(read_role(e, address), Some(RoleType::SuperAdmin))
}

pub fn is_verifier(e: &Env, address: &Address) -> bool {
    matches!(read_role(e, address), Some(RoleType::Verifier))
}

pub fn is_trader(e: &Env, address: &Address) -> bool {
    matches!(read_role(e, address), Some(RoleType::Trader))
}
use soroban_sdk::{contracttype, Address, Env};

// ── TTL Constants ──────────────────────────────────────────────────────────────
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280;  // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32        = 518400;  // ~30 days

// ── Role Type ──────────────────────────────────────────────────────────────────
/// Role types in the FarmCredit ecosystem.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoleType {
    /// SuperAdmin - controls platform parameters, can add/remove verifiers.
    SuperAdmin = 0,
    /// Verifier - agricultural oracles that verify real-world data.
    Verifier   = 1,
    /// Trader - participants who can trade carbon credits.
    Trader     = 2,
}

// ── Storage Keys ───────────────────────────────────────────────────────────────
/// Internal storage keys for the RBAC contract.
/// Named `StorageKey` to avoid collision with the public `DataKey` in lib.rs.
#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    /// Boolean flag — true once initialize() has been called.
    Initialized,
    /// The current SuperAdmin address.
    SuperAdmin,
    /// Mapping from Address → RoleType (persistent, per-address).
    Role(Address),
    /// Presence flag for Verifier role (enables efficient membership checks).
    Verifiers(Address),
    /// Presence flag for Trader role.
    Traders(Address),
}

// ── Initialization ─────────────────────────────────────────────────────────────

/// Returns true if the contract has been initialized.
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Initialized)
}

/// Marks the contract as initialized.
pub fn set_initialized(env: &Env) {
    env.storage()
        .instance()
        .set(&StorageKey::Initialized, &true);
}

// ── SuperAdmin ─────────────────────────────────────────────────────────────────

/// Returns true if a SuperAdmin address has been written.
pub fn has_super_admin(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::SuperAdmin)
}

/// Returns the SuperAdmin address. Panics with a clear message if not set.
pub fn read_super_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&StorageKey::SuperAdmin)
        .expect("super admin not set: was initialize() called?")
}

/// Writes the SuperAdmin address to instance storage.
pub fn write_super_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&StorageKey::SuperAdmin, admin);
}

// ── Role Helpers ───────────────────────────────────────────────────────────────

/// Returns the role of an address, or None if no role is assigned.
pub fn read_role(env: &Env, address: &Address) -> Option<RoleType> {
    env.storage()
        .persistent()
        .get(&StorageKey::Role(address.clone()))
}

/// Removes the role entry for an address from persistent storage.
pub fn remove_role(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::Role(address.clone()));
}

// ── SuperAdmin Role ────────────────────────────────────────────────────────────

/// Grants the SuperAdmin role to an address.
pub fn grant_super_admin(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::SuperAdmin);
}

/// Revokes the SuperAdmin role from an address.
/// Used by transfer_admin() to clear the outgoing admin's role.
pub fn revoke_super_admin(env: &Env, address: &Address) {
    remove_role(env, address);
}

// ── Verifier Role ──────────────────────────────────────────────────────────────

/// Grants the Verifier role to an address.
pub fn grant_verifier(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::Verifier);
    env.storage()
        .instance()
        .set(&StorageKey::Verifiers(address.clone()), &true);
}

/// Revokes the Verifier role from an address.
pub fn revoke_verifier(env: &Env, address: &Address) {
    remove_role(env, address);
    env.storage()
        .instance()
        .remove(&StorageKey::Verifiers(address.clone()));
}

// ── Trader Role ────────────────────────────────────────────────────────────────

/// Grants the Trader role to an address.
pub fn grant_trader(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::Trader);
    env.storage()
        .instance()
        .set(&StorageKey::Traders(address.clone()), &true);
}

/// Revokes the Trader role from an address.
pub fn revoke_trader(env: &Env, address: &Address) {
    remove_role(env, address);
    env.storage()
        .instance()
        .remove(&StorageKey::Traders(address.clone()));
}

// ── Role Checks ────────────────────────────────────────────────────────────────

/// Returns true if the address holds the SuperAdmin role.
pub fn is_super_admin(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::SuperAdmin))
}

/// Returns true if the address holds the Verifier role.
pub fn is_verifier(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::Verifier))
}

/// Returns true if the address holds the Trader role.
pub fn is_trader(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::Trader))
}