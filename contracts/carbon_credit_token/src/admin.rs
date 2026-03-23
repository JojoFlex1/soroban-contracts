use soroban_sdk::{Address, Env};

use crate::storage::DataKey;

pub fn has_administrator(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Admin)
}

pub fn read_administrator(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    e.storage().instance().set(&DataKey::Admin, id);
}

// ── SuperAdmin ────────────────────────────────────────────────────────────────

pub fn has_super_admin(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::SuperAdmin)
}

pub fn read_super_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::SuperAdmin).unwrap()
}

pub fn write_super_admin(e: &Env, id: &Address) {
    e.storage().instance().set(&DataKey::SuperAdmin, id);
}

// ── Verifier role ─────────────────────────────────────────────────────────────

pub fn is_verifier(e: &Env, addr: &Address) -> bool {
    e.storage()
        .instance()
        .get::<DataKey, bool>(&DataKey::Verifier(addr.clone()))
        .unwrap_or(false)
}

pub fn grant_verifier(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::Verifier(addr.clone()), &true);
}

pub fn revoke_verifier(e: &Env, addr: &Address) {
    e.storage()
        .instance()
        .remove(&DataKey::Verifier(addr.clone()));
}

// ── Blacklist ─────────────────────────────────────────────────────────────────

pub fn is_blacklisted(e: &Env, addr: &Address) -> bool {
    e.storage()
        .instance()
        .get::<DataKey, bool>(&DataKey::Blacklisted(addr.clone()))
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
