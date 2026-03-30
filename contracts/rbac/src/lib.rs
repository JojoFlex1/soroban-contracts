#![no_std]

mod error;
mod storage;

pub use error::Error;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use storage::{
    is_admin, is_super_admin, is_trader, is_verifier, read_role, read_super_admin,
    write_admin, write_role, write_super_admin, RoleType,
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if storage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_initialized(&env);
        write_super_admin(&env, &admin);
        write_admin(&env, &admin);
        write_role(&env, &admin, RoleType::SuperAdmin);
        Ok(())
    }

    // --- Role Grant ---

    pub fn grant_admin(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        if let Some(existing) = read_role(&env, &account) {
            if existing != RoleType::Admin {
                return Err(Error::AddressHasDifferentRole);
            }
            return Err(Error::RoleAlreadyAssigned);
        }
        write_admin(&env, &account);
        write_role(&env, &account, RoleType::Admin);
        Ok(())
    }

    pub fn grant_verifier(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        if let Some(existing) = read_role(&env, &account) {
            if existing != RoleType::Verifier {
                return Err(Error::AddressHasDifferentRole);
            }
            return Err(Error::RoleAlreadyAssigned);
        }
        write_role(&env, &account, RoleType::Verifier);
        Ok(())
    }

    pub fn grant_trader(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        if let Some(existing) = read_role(&env, &account) {
            if existing != RoleType::Trader {
                return Err(Error::AddressHasDifferentRole);
            }
            return Err(Error::RoleAlreadyAssigned);
        }
        write_role(&env, &account, RoleType::Trader);
        Ok(())
    }

    pub fn revoke_role(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        if !is_admin(&env, &admin) {
            return Err(Error::Unauthorized);
        }
        if is_super_admin(&env, &account) {
            return Err(Error::CannotRemoveSuperAdmin);
        }
        match read_role(&env, &account) {
            Some(RoleType::Admin) => {
                storage::revoke_admin(&env, &account);
                storage::remove_role(&env, &account);
                Ok(())
            }
            Some(RoleType::Verifier) => {
                storage::revoke_verifier(&env, &account);
                Ok(())
            }
            Some(RoleType::Trader) => {
                storage::revoke_trader(&env, &account);
                Ok(())
            }
            Some(RoleType::SuperAdmin) => Err(Error::CannotRemoveSuperAdmin),
            None => Err(Error::RoleNotAssigned),
        }
    }

    // --- Convenience wrappers (used by tests and cross-contract callers) ---

    pub fn add_verifier(env: Env, account: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();
        if let Some(existing) = read_role(&env, &account) {
            if existing != RoleType::Verifier {
                return Err(Error::AddressHasDifferentRole);
            }
            return Err(Error::RoleAlreadyAssigned);
        }
        write_role(&env, &account, RoleType::Verifier);
        Ok(())
    }

    pub fn add_trader(env: Env, account: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();
        if let Some(existing) = read_role(&env, &account) {
            if existing != RoleType::Trader {
                return Err(Error::AddressHasDifferentRole);
            }
            return Err(Error::RoleAlreadyAssigned);
        }
        write_role(&env, &account, RoleType::Trader);
        Ok(())
    }

    pub fn remove_verifier(env: Env, account: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();
        if !is_verifier(&env, &account) {
            return Err(Error::RoleNotAssigned);
        }
        storage::revoke_verifier(&env, &account);
        Ok(())
    }

    pub fn remove_trader(env: Env, account: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();
        if !is_trader(&env, &account) {
            return Err(Error::RoleNotAssigned);
        }
        storage::revoke_trader(&env, &account);
        Ok(())
    }

    // --- Role Checks ---

    pub fn has_role(env: Env, account: Address, role: String) -> bool {
        let role_type = if role == String::from_str(&env, "Admin") {
            RoleType::Admin
        } else if role == String::from_str(&env, "Verifier") {
            RoleType::Verifier
        } else if role == String::from_str(&env, "Trader") {
            RoleType::Trader
        } else if role == String::from_str(&env, "SuperAdmin") {
            RoleType::SuperAdmin
        } else {
            return false;
        };
        if let Some(assigned) = read_role(&env, &account) {
            assigned == role_type
        } else {
            false
        }
    }

    pub fn is_admin(env: Env, account: Address) -> bool {
        is_admin(&env, &account)
    }

    pub fn is_verifier(env: Env, account: Address) -> bool {
        is_verifier(&env, &account)
    }

    pub fn is_trader(env: Env, account: Address) -> bool {
        is_trader(&env, &account)
    }

    pub fn is_super_admin(env: Env, account: Address) -> bool {
        is_super_admin(&env, &account)
    }

    pub fn get_super_admin(env: Env) -> Address {
        read_super_admin(&env)
    }

    pub fn get_role(env: Env, account: Address) -> u32 {
        match read_role(&env, &account) {
            Some(RoleType::SuperAdmin) => 0,
            Some(RoleType::Verifier)   => 1,
            Some(RoleType::Trader)     => 2,
            Some(RoleType::Admin)      => 3,
            None                       => 255,
        }
    }

    // --- Admin Transfer ---

    pub fn transfer_admin(env: Env, old_admin: Address, new_admin: Address) -> Result<(), Error> {
        old_admin.require_auth();
        if !is_super_admin(&env, &old_admin) {
            return Err(Error::Unauthorized);
        }
        write_super_admin(&env, &new_admin);
        write_admin(&env, &new_admin);
        write_role(&env, &new_admin, RoleType::SuperAdmin);
        storage::remove_role(&env, &old_admin);
        storage::revoke_admin(&env, &old_admin);
        Ok(())
    }
}

#[cfg(test)]
mod test;
