use soroban_sdk::{contracttype, Address, Env};

// ── TTL Constants ──────────────────────────────────────────────────────────────
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400;        // ~30 days
pub const BALANCE_LIFETIME_THRESHOLD: u32 = 17280;  // ~1 day
pub const BALANCE_BUMP_AMOUNT: u32 = 518400;         // ~30 days

// ── Allowance Types ────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[derive(Clone)]
#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

// ── Storage Keys ───────────────────────────────────────────────────────────────
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    /// Address of the external RBAC contract used for role verification.
    RbacContract,
    /// SuperAdmin address stored inline (used by token contract admin checks).
    SuperAdmin,
    /// Per-address verifier flag (used when RBAC is managed internally).
    Verifier(Address),
    /// Per-address blacklist flag.
    Blacklisted(Address),
    Balance(Address),
    Allowance(AllowanceDataKey),
    Name,
    Symbol,
    Decimals,
    TotalSupply,
    TotalRetired,
    Initialized,
}

// ── Initialization ─────────────────────────────────────────────────────────────
pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(e: &Env) {
    e.storage().instance().set(&DataKey::Initialized, &true);
}

// ── RBAC Contract ──────────────────────────────────────────────────────────────
/// Persists the external RBAC contract address used for role-based minting checks.
pub fn write_rbac_contract(e: &Env, rbac_id: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::RbacContract, rbac_id);
}

/// Reads the registered RBAC contract address.
///
/// Panics with a clear diagnostic if the contract has not been initialised.
pub fn read_rbac_contract(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::RbacContract)
        .expect("rbac contract address not set: was initialize() called?")
}

// ── Administrator ──────────────────────────────────────────────────────────────
pub fn read_administrator(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("administrator not set")
}

pub fn write_administrator(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn read_super_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::SuperAdmin)
        .expect("super admin not set")
}

pub fn write_super_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, admin);
}

// ── Verifier / Blacklist (inline RBAC) ────────────────────────────────────────
pub fn grant_verifier(e: &Env, verifier: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::Verifier(verifier.clone()), &true);
}

pub fn revoke_verifier(e: &Env, verifier: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Verifier(verifier.clone()));
}

pub fn is_verifier(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Verifier(addr.clone()))
        .unwrap_or(false)
}

pub fn blacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::Blacklisted(addr.clone()), &true);
}

pub fn unblacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Blacklisted(addr.clone()));
}

pub fn is_blacklisted(e: &Env, addr: &Address) -> bool {
    e.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Blacklisted(addr.clone()))
        .unwrap_or(false)
}

// ── Supply Accounting ──────────────────────────────────────────────────────────
pub fn read_total_supply(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn write_total_supply(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalSupply, &amount);
}

pub fn read_total_retired(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalRetired)
        .unwrap_or(0)
}

pub fn write_total_retired(e: &Env, amount: i128) {
    e.storage().instance().set(&DataKey::TotalRetired, &amount);
}