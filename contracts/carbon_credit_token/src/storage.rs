use soroban_sdk::{contracttype, Address, Env};

// TTL Constants
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days
pub const BALANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const BALANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days

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

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    /// Address of the external RBAC contract used for role verification.
    RbacContract,
    Balance(Address),
    Allowance(AllowanceDataKey),
    Name,
    Symbol,
    Decimals,
    TotalSupply,
    TotalRetired,
    Initialized,
}

pub fn is_initialized(e: &Env) -> bool {
    let key = DataKey::Initialized;
    e.storage().instance().has(&key)
}

pub fn set_initialized(e: &Env) {
    let key = DataKey::Initialized;
    e.storage().instance().set(&key, &true);
}

// ── RBAC Contract ──────────────────────────────────────────────────────────────

/// Persists the RBAC contract address used for role-based minting checks.
pub fn write_rbac_contract(e: &Env, rbac_id: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::RbacContract, rbac_id);
}

/// Reads the registered RBAC contract address.
///
/// Panics if the contract has not been initialised, providing a clear
/// diagnostic rather than an opaque unwrap failure.
pub fn read_rbac_contract(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::RbacContract)
        .expect("rbac contract address not set: was initialize() called?")
}

// ── Supply Accounting ──────────────────────────────────────────────────────────

pub fn read_total_supply(e: &Env) -> i128 {
    let key = DataKey::TotalSupply;
    e.storage().instance().get(&key).unwrap_or(0)
}

pub fn write_total_supply(e: &Env, amount: i128) {
    let key = DataKey::TotalSupply;
    e.storage().instance().set(&key, &amount);
}

pub fn read_total_retired(e: &Env) -> i128 {
    let key = DataKey::TotalRetired;
    e.storage().instance().get(&key).unwrap_or(0)
}

pub fn write_total_retired(e: &Env, amount: i128) {
    let key = DataKey::TotalRetired;
    e.storage().instance().set(&key, &amount);
}