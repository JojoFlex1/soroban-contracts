use soroban_sdk::{contracttype, Address, Env};

pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280;  // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32        = 518400;  // ~30 days

// ── Role Type ──────────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoleType {
    SuperAdmin,
    Verifier,
    Trader,
}

// ── Storage Keys ───────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Initialized,
    SuperAdmin,
    Role(Address),
}

// ── Initialization ─────────────────────────────────────────────────────────────
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&StorageKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage()
        .instance()
        .set(&StorageKey::Initialized, &true);
}

// ── SuperAdmin ─────────────────────────────────────────────────────────────────
pub fn read_super_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&StorageKey::SuperAdmin)
        .expect("super admin not set: was initialize() called?")
}

pub fn write_super_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .set(&StorageKey::SuperAdmin, admin);
}

// ── Role Helpers ───────────────────────────────────────────────────────────────
pub fn read_role(env: &Env, address: &Address) -> Option<RoleType> {
    env.storage()
        .persistent()
        .get(&StorageKey::Role(address.clone()))
}

pub fn grant_super_admin(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::SuperAdmin);
}

pub fn revoke_super_admin(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::Role(address.clone()));
}

pub fn grant_verifier(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::Verifier);
}

pub fn revoke_verifier(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::Role(address.clone()));
}

pub fn grant_trader(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .set(&StorageKey::Role(address.clone()), &RoleType::Trader);
}

pub fn revoke_trader(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&StorageKey::Role(address.clone()));
}

// ── Role Checks ────────────────────────────────────────────────────────────────
pub fn is_super_admin(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::SuperAdmin))
}

pub fn is_verifier(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::Verifier))
}

pub fn is_trader(env: &Env, address: &Address) -> bool {
    matches!(read_role(env, address), Some(RoleType::Trader))
}