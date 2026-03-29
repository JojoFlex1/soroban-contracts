use soroban_sdk::{contracttype, Address, Bytes, Env, Vec};

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct OffsetCertificate {
    pub id: u64,
    pub amount: i128,
    pub timestamp: u64,
}


// ── TTL Constants ──────────────────────────────────────────────────────────────
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const INSTANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days

pub const BALANCE_LIFETIME_THRESHOLD: u32 = 17280; // ~1 day
pub const BALANCE_BUMP_AMOUNT: u32 = 518400; // ~30 days

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
    // Admin / roles
    RbacContract,
    Admin,
    SuperAdmin,
    Verifier(Address),
    Blacklisted(Address),

    // Ledger/accounting
    Balance(Address),
    Allowance(AllowanceDataKey),
    TotalSupply,
    TotalRetired,

    // Metadata
    Name,
    Symbol,
    Decimals,

    // Init flag
    Initialized,
    VerifierRegistry,
    UsedReportHash(Bytes),

    // Offset Certificates
    CertificateCount,
    Certificates(Address),
}


// ── Initialization ─────────────────────────────────────────────────────────────
pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn set_initialized(e: &Env) {
    e.storage()
        .instance()
        .set(&DataKey::Initialized, &true);
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
        .instance()
        .set(&DataKey::Verifier(verifier.clone()), &true);
}

pub fn revoke_verifier(e: &Env, verifier: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::Verifier(verifier.clone()));
}

pub fn is_verifier(e: &Env, addr: &Address) -> bool {
    e.storage()
        .instance()
        .get::<DataKey, bool>(&DataKey::Verifier(addr.clone()))
        .unwrap_or(false)
}

pub fn blacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::Blacklisted(addr.clone()), &true);
}

pub fn unblacklist_address(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::Blacklisted(addr.clone()));
}

pub fn is_blacklisted(e: &Env, addr: &Address) -> bool {
    e.storage()
        .instance()
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
    e.storage()
        .instance()
        .set(&DataKey::TotalSupply, &amount);
}

pub fn read_total_retired(e: &Env) -> i128 {
    e.storage()
        .instance()
        .get(&DataKey::TotalRetired)
        .unwrap_or(0)
}

pub fn write_total_retired(e: &Env, amount: i128) {
    e.storage()
        .instance()
        .set(&DataKey::TotalRetired, &amount);
}

// ── Offset Certificates ────────────────────────────────────────────────────────
pub fn read_certificate_count(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get(&DataKey::CertificateCount)
        .unwrap_or(0)
}

pub fn increment_certificate_count(e: &Env) -> u64 {
    let count = read_certificate_count(e) + 1;
    e.storage()
        .instance()
        .set(&DataKey::CertificateCount, &count);
    count
}

pub fn read_certificates(e: &Env, corporate: Address) -> Vec<OffsetCertificate> {
    e.storage()
        .persistent()
        .get(&DataKey::Certificates(corporate))
        .unwrap_or_else(|| Vec::new(e))
}

pub fn write_certificate(e: &Env, corporate: Address, cert: OffsetCertificate) {
    let mut certs = read_certificates(e, corporate.clone());
    certs.push_back(cert);
    e.storage()
        .persistent()
        .set(&DataKey::Certificates(corporate.clone()), &certs);


    // Bump TTL for persistent storage
    e.storage()
        .persistent()
        .extend_ttl(&DataKey::Certificates(corporate.clone()), 17280, 518400);
}

