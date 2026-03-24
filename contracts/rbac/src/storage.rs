use soroban_sdk::{contracttype, Address, Env};

// TTL Constants
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days

/// Role types in the FarmCredit ecosystem
#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum RoleType {
    /// SuperAdmin - controls platform parameters, can add/remove verifiers
    SuperAdmin = 0,
    /// Verifier - agricultural oracles that verify real-world data
    Verifier = 1,
    /// Trader - participants who can trade carbon credits
    Trader = 2,
}

/// Storage keys for RBAC contract
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Boolean flag to check if contract is initialized
    Initialized,
    /// The SuperAdmin address
    SuperAdmin,
    /// Mapping from Address to RoleType
    AddressRole(Address),
    /// Set of addresses with Verifier role (for efficient iteration if needed)
    Verifiers(Address),
    /// Set of addresses with Trader role
    Traders(Address),
}

/// Check if the contract has been initialized
pub fn is_initialized(e: &Env) -> bool {
    let key = DataKey::Initialized;
    e.storage().instance().has(&key)
}

/// Mark the contract as initialized
pub fn set_initialized(e: &Env) {
    let key = DataKey::Initialized;
    e.storage().instance().set(&key, &true);
}

/// Check if SuperAdmin exists
pub fn has_super_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::SuperAdmin)
}

/// Read the SuperAdmin address
pub fn read_super_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get(&DataKey::SuperAdmin)
        .unwrap()
}

/// Write the SuperAdmin address
pub fn write_super_admin(e: &Env, id: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, id);
}

/// Grant SuperAdmin role to an address
pub fn grant_super_admin(e: &Env, addr: &Address) {
    write_role(e, addr, RoleType::SuperAdmin);
}

/// Read the role of an address
pub fn read_role(e: &Env, addr: &Address) -> Option<RoleType> {
    e.storage()
        .instance()
        .get::<DataKey, RoleType>(&DataKey::AddressRole(addr.clone()))
}

/// Write the role for an address
pub fn write_role(e: &Env, addr: &Address, role: RoleType) {
    e.storage()
        .instance()
        .set(&DataKey::AddressRole(addr.clone()), &role);
}

/// Remove the role from an address
pub fn remove_role(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::AddressRole(addr.clone()));
}

/// Check if an address is a Verifier
pub fn is_verifier(e: &Env, addr: &Address) -> bool {
    read_role(e, addr) == Some(RoleType::Verifier)
}

/// Add a Verifier role to an address
pub fn grant_verifier(e: &Env, addr: &Address) {
    write_role(e, addr, RoleType::Verifier);
    e.storage()
        .instance()
        .set(&DataKey::Verifiers(addr.clone()), &true);
}

/// Revoke Verifier role from an address
pub fn revoke_verifier(e: &Env, addr: &Address) {
    remove_role(e, addr);
    e.storage()
        .instance()
        .remove(&DataKey::Verifiers(addr.clone()));
}

/// Check if an address is a Trader
pub fn is_trader(e: &Env, addr: &Address) -> bool {
    read_role(e, addr) == Some(RoleType::Trader)
}

/// Add a Trader role to an address
pub fn grant_trader(e: &Env, addr: &Address) {
    write_role(e, addr, RoleType::Trader);
    e.storage()
        .instance()
        .set(&DataKey::Traders(addr.clone()), &true);
}

/// Revoke Trader role from an address
pub fn revoke_trader(e: &Env, addr: &Address) {
    remove_role(e, addr);
    e.storage()
        .instance()
        .remove(&DataKey::Traders(addr.clone()));
}

/// Check if an address is a SuperAdmin
pub fn is_super_admin(e: &Env, addr: &Address) -> bool {
    read_role(e, addr) == Some(RoleType::SuperAdmin)
}