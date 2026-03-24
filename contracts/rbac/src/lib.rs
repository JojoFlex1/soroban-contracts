#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Role(Address),
}

pub const ROLE_VERIFIER: Symbol = symbol_short!("VERIFIER");
pub const ROLE_ADMIN: Symbol    = symbol_short!("ADMIN");
pub const ROLE_NONE: Symbol     = symbol_short!("NONE");

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Role(admin), &ROLE_ADMIN);
    }

    pub fn grant_verifier(env: Env, admin: Address, verifier: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::Role(verifier), &ROLE_VERIFIER);
    }

    pub fn revoke_role(env: Env, admin: Address, account: Address) {
        admin.require_auth();
        Self::require_admin(&env, &admin);
        env.storage().persistent().remove(&DataKey::Role(account));
    }

    pub fn get_role(env: Env, account: Address) -> Symbol {
        env.storage()
            .persistent()
            .get::<DataKey, Symbol>(&DataKey::Role(account))
            .unwrap_or(ROLE_NONE)
    }

    pub fn is_verifier(env: Env, account: Address) -> bool {
        Self::get_role(env, account) == ROLE_VERIFIER
    }

    pub fn is_admin(env: Env, account: Address) -> bool {
        Self::get_role(env, account) == ROLE_ADMIN
    }

    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        Self::require_admin(&env, &current_admin);
        env.storage().persistent().remove(&DataKey::Role(current_admin));
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().set(&DataKey::Role(new_admin), &ROLE_ADMIN);
    }

    fn require_admin(env: &Env, caller: &Address) {
        let stored_admin: Address = env
            .storage().instance()
            .get(&DataKey::Admin)
            .expect("not initialized");
        if *caller != stored_admin {
            panic!("unauthorized: caller is not admin");
        }
    }
}
