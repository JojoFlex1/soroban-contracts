#![no_std]

mod error;
mod storage;

use error::Error;
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

use storage::{
    is_initialized, is_super_admin, is_trader, is_verifier, read_role, read_super_admin,
    revoke_trader, revoke_verifier, set_initialized, write_super_admin, INSTANCE_BUMP_AMOUNT,
    INSTANCE_LIFETIME_THRESHOLD, RoleType,
};

// ── Role Symbol Constants ──────────────────────────────────────────────────────
pub const ROLE_VERIFIER: Symbol = symbol_short!("VERIFIER");
pub const ROLE_ADMIN: Symbol    = symbol_short!("ADMIN");
pub const ROLE_TRADER: Symbol   = symbol_short!("TRADER");
pub const ROLE_NONE: Symbol     = symbol_short!("NONE");

// ── DataKey ────────────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    SuperAdmin,
    Role(Address),
    Initialized,
}

#[contract]
pub struct RbacContract;

#[contractimpl]
impl RbacContract {
    // ── Initialization ─────────────────────────────────────────────────────────

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

    // ── View Functions ─────────────────────────────────────────────────────────

    /// Returns the role of the given address as a u32:
    /// 0 = SuperAdmin, 1 = Verifier, 2 = Trader, 255 = None
    pub fn get_role(env: Env, address: Address) -> u32 {
        match read_role(&env, &address) {
            Some(RoleType::SuperAdmin) => 0,
            Some(RoleType::Verifier)   => 1,
            Some(RoleType::Trader)     => 2,
            None                       => 255,
        }
    }

    /// Returns the role of `account` as a Symbol for cross-contract compatibility.
    pub fn get_role_symbol(env: Env, account: Address) -> Symbol {
        match read_role(&env, &account) {
            Some(RoleType::SuperAdmin) => ROLE_ADMIN,
            Some(RoleType::Verifier)   => ROLE_VERIFIER,
            Some(RoleType::Trader)     => ROLE_TRADER,
            None                       => ROLE_NONE,
        }
    }

    /// Returns `true` if `address` has the Verifier role.
    /// This is the method called cross-contract by the token contract.
    pub fn has_role(env: Env, address: Address, role: soroban_sdk::String) -> bool {
        use soroban_sdk::String as SString;
        let verifier_str = SString::from_str(&env, "Verifier");
        let admin_str    = SString::from_str(&env, "Admin");
        let trader_str   = SString::from_str(&env, "Trader");

        match read_role(&env, &address) {
            Some(RoleType::Verifier)   => role == verifier_str,
            Some(RoleType::SuperAdmin) => role == admin_str,
            Some(RoleType::Trader)     => role == trader_str,
            None                       => false,
        }
    }

    /// Returns `true` if the address has the SuperAdmin role.
    pub fn is_super_admin(env: Env, address: Address) -> bool {
        is_super_admin(&env, &address)
    }

    /// Returns `true` if the address has the Verifier role.
    pub fn is_verifier(env: Env, address: Address) -> bool {
        is_verifier(&env, &address)
    }

    /// Returns `true` if the address has the Trader role.
    pub fn is_trader(env: Env, address: Address) -> bool {
        is_trader(&env, &address)
    }

    /// Returns the SuperAdmin address.
    pub fn get_super_admin(env: Env) -> Address {
        read_super_admin(&env)
    }

    // ── Admin Functions (SuperAdmin only) ──────────────────────────────────────

    /// Grants the Verifier role to an address.
    /// Only the SuperAdmin can call this.
    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if let Some(existing_role) = read_role(&env, &verifier) {
            if existing_role == RoleType::Verifier {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        storage::grant_verifier(&env, &verifier);
        Ok(())
    }

    /// Grants the Verifier role to an address (alias matching legacy interface).
    pub fn grant_verifier(env: Env, admin: Address, verifier: Address) -> Result<(), Error> {
        admin.require_auth();
        let super_admin = read_super_admin(&env);
        if admin != super_admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if let Some(existing_role) = read_role(&env, &verifier) {
            if existing_role == RoleType::Verifier {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        storage::grant_verifier(&env, &verifier);
        Ok(())
    }

    /// Removes the Verifier role from an address.
    /// Only the SuperAdmin can call this.
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

    /// Removes any role from an address (legacy `revoke_role` interface).
    pub fn revoke_role(env: Env, admin: Address, account: Address) -> Result<(), Error> {
        admin.require_auth();
        let super_admin = read_super_admin(&env);
        if admin != super_admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        match read_role(&env, &account) {
            Some(RoleType::Verifier) => revoke_verifier(&env, &account),
            Some(RoleType::Trader)   => revoke_trader(&env, &account),
            Some(RoleType::SuperAdmin) => return Err(Error::CannotRevokeAdmin),
            None => return Err(Error::RoleNotAssigned),
        }

        Ok(())
    }

    /// Grants the Trader role to an address.
    /// Only the SuperAdmin can call this.
    pub fn add_trader(env: Env, trader: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        if let Some(existing_role) = read_role(&env, &trader) {
            if existing_role == RoleType::Trader {
                return Err(Error::RoleAlreadyAssigned);
            }
            return Err(Error::AddressHasDifferentRole);
        }

        storage::grant_trader(&env, &trader);
        Ok(())
    }

    /// Removes the Trader role from an address.
    /// Only the SuperAdmin can call this.
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
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) -> Result<(), Error> {
        current_admin.require_auth();
        let super_admin = read_super_admin(&env);
        if current_admin != super_admin {
            return Err(Error::Unauthorized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        // Revoke old admin's role, grant to new one
        storage::revoke_super_admin(&env, &current_admin);
        write_super_admin(&env, &new_admin);
        storage::grant_super_admin(&env, &new_admin);

        Ok(())
    }
}

mod test;