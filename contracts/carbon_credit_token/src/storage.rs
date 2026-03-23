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
    SuperAdmin,
    Verifier(Address),
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

pub fn is_initialized(e: &Env) -> bool {
    let key = DataKey::Initialized;
    e.storage().instance().has(&key)
}

pub fn set_initialized(e: &Env) {
    let key = DataKey::Initialized;
    e.storage().instance().set(&key, &true);
}

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
