#![no_std]

mod error;
mod storage;

use error::Error;
use soroban_sdk::{contract, contractimpl, Address, Env};

use storage::{
    is_initialized, is_super_admin, is_trader, is_verifier, read_role, read_super_admin,
    revoke_trader, revoke_verifier, set_initialized, write_super_admin, INSTANCE_BUMP_AMOUNT,
    INSTANCE_LIFETIME_THRESHOLD, RoleType,
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
        storage::grant_super_admin(&env, &super_admin);

        Ok(())
    }

    // ── View functions ───────────────────────────────────────────────────────────

    /// Returns the role of the given address as a u32.
    /// 0 = SuperAdmin, 1 = Verifier, 2 = Trader, 255 = None
    pub fn get_role(env: Env, address: Address) -> u32 {
        match read_role(&env, &address) {
            Some(RoleType::SuperAdmin) => 0,
            Some(RoleType::Verifier) => 1,
            Some(RoleType::Trader) => 2,
            None => 255,
        }
    }

    /// Returns true if the address has SuperAdmin role.
    pub fn is_super_admin(env: Env, address: Address) -> bool {
        is_super_admin(&env, &address)
    }

    /// Returns true if the address has Verifier role.
    pub fn is_verifier(env: Env, address: Address) -> bool {
        is_verifier(&env, &address)
    }

    /// Returns true if the address has Trader role.
    pub fn is_trader(env: Env, address: Address) -> bool {
        is_trader(&env, &address)
    }

    /// Returns the SuperAdmin address.
    pub fn get_super_admin(env: Env) -> Address {
        read_super_admin(&env)
    }

    // ── Admin functions (SuperAdmin only) ─────────────────────────────────────────

    /// Adds a Verifier role to an address.
    /// Only the SuperAdmin can call this.
    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if already has a role
        if let Some(existing_role) = read_role(&env, &verifier) {
            if existing_role == RoleType::Verifier {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        // Grant the verifier role
        storage::grant_verifier(&env, &verifier);
        Ok(())
    }

    /// Removes the Verifier role from an address.
    /// Only the SuperAdmin can call this.
    pub fn remove_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if the address is actually a verifier
        if !is_verifier(&env, &verifier) {
            return Err(Error::RoleNotAssigned);
        }

        revoke_verifier(&env, &verifier);
        Ok(())
    }

    /// Adds a Trader role to an address.
    /// Only the SuperAdmin can call this.
    pub fn add_trader(env: Env, trader: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if already has a role
        if let Some(existing_role) = read_role(&env, &trader) {
            if existing_role == RoleType::Trader {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        // Grant the trader role
        storage::grant_trader(&env, &trader);
        Ok(())
    }

    /// Removes the Trader role from an address.
    /// Only the SuperAdmin can call this.
    pub fn remove_trader(env: Env, trader: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        // Extend TTL for storage
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Check if the address is actually a trader
        if !is_trader(&env, &trader) {
            return Err(Error::RoleNotAssigned);
        }

        revoke_trader(&env, &trader);
        Ok(())
    }
}

mod test;