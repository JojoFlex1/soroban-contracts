#![no_std]

mod error;
mod storage;

pub use error::Error;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use storage::{
    grant_super_admin, grant_trader, grant_verifier, is_trader, is_verifier, read_role,
    read_super_admin, revoke_super_admin, revoke_trader, revoke_verifier, set_initialized,
    write_super_admin, is_initialized, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    /// Initializes the contract with the initial SuperAdmin.
    /// Can only be called once.
    pub fn initialize(env: Env, super_admin: Address) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_initialized(&env);
        write_super_admin(&env, &super_admin);
        grant_super_admin(&env, &super_admin);
        Ok(())
    }

    /// Returns the role of `address` as a u32:
    /// 0 = SuperAdmin, 1 = Verifier, 2 = Trader, 255 = None
    pub fn get_role(env: Env, address: Address) -> u32 {
        match read_role(&env, &address) {
            Some(storage::RoleType::SuperAdmin) => 0,
            Some(storage::RoleType::Verifier) => 1,
            Some(storage::RoleType::Trader) => 2,
            None => 255,
        }
    }

    /// Returns the SuperAdmin address.
    pub fn get_super_admin(env: Env) -> Address {
        read_super_admin(&env)
    }

    pub fn is_super_admin(env: Env, address: Address) -> bool {
        storage::is_super_admin(&env, &address)
    }

    pub fn is_verifier(env: Env, address: Address) -> bool {
        is_verifier(&env, &address)
    }

    pub fn is_trader(env: Env, address: Address) -> bool {
        is_trader(&env, &address)
    }

    /// Cross-contract interface: checks if `address` has `role`.
    /// Expected roles: "Verifier", "Admin", "Trader".
    pub fn has_role(env: Env, address: Address, role: String) -> bool {
        let verifier_str = String::from_str(&env, "Verifier");
        let admin_str = String::from_str(&env, "Admin");
        let trader_str = String::from_str(&env, "Trader");

        match read_role(&env, &address) {
            Some(storage::RoleType::Verifier) => role == verifier_str,
            Some(storage::RoleType::SuperAdmin) => role == admin_str,
            Some(storage::RoleType::Trader) => role == trader_str,
            None => false,
        }
    }

    /// Adds a Verifier role to an address.
    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if let Some(existing) = read_role(&env, &verifier) {
            if existing == storage::RoleType::Verifier {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        grant_verifier(&env, &verifier);
        Ok(())
    }

    /// Removes the Verifier role from an address.
    pub fn remove_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if !is_verifier(&env, &verifier) {
            return Err(Error::RoleNotAssigned);
        }

        revoke_verifier(&env, &verifier);
        Ok(())
    }

    /// Adds a Trader role to an address.
    pub fn add_trader(env: Env, trader: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if let Some(existing) = read_role(&env, &trader) {
            if existing == storage::RoleType::Trader {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        grant_trader(&env, &trader);
        Ok(())
    }

    /// Removes the Trader role from an address.
    pub fn remove_trader(env: Env, trader: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if !is_trader(&env, &trader) {
            return Err(Error::RoleNotAssigned);
        }

        revoke_trader(&env, &trader);
        Ok(())
    }

    /// Transfers the SuperAdmin role to a new address.
    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), Error> {
        current_admin.require_auth();

        let super_admin = read_super_admin(&env);
        if current_admin != super_admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        revoke_super_admin(&env, &current_admin);
        write_super_admin(&env, &new_admin);
        grant_super_admin(&env, &new_admin);
        Ok(())
    }

    /// Legacy interface: removes any role from `account`.
    /// Only the SuperAdmin may call this.
    pub fn revoke_role(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();

        let super_admin = read_super_admin(&env);
        if admin != super_admin {
            return Err(Error::Unauthorized);
        }

        if storage::is_super_admin(&env, &account) {
            return Err(Error::CannotRemoveSuperAdmin);
        }

        match read_role(&env, &account) {
            Some(storage::RoleType::Verifier) => {
                revoke_verifier(&env, &account);
                Ok(())
            }
            Some(storage::RoleType::Trader) => {
                revoke_trader(&env, &account);
                Ok(())
            }
            Some(storage::RoleType::SuperAdmin) => Err(Error::CannotRemoveSuperAdmin),
            None => Err(Error::RoleNotAssigned),
        }

    }
}