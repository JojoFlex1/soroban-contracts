#![no_std]

mod admin;
mod allowance;
mod balance;
mod error;
mod events;
mod metadata;
mod storage;
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::admin::{
    blacklist_address, grant_verifier, is_blacklisted, is_verifier, read_administrator,
    read_super_admin, revoke_verifier, unblacklist_address, write_administrator, write_super_admin,
};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::error::Error;
use crate::events::{ApproveEvent, BurnEvent, MintEvent, RetirementEvent, TransferEvent};
use crate::metadata::{read_decimals, read_name, read_symbol, write_metadata};
use crate::storage::{
    is_initialized, read_total_retired, read_total_supply, set_initialized, write_total_retired,
    write_total_supply, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

fn check_nonnegative_amount(amount: i128) -> Result<(), Error> {
    if amount < 0 {
        Err(Error::NegativeAmount)
    } else {
        Ok(())
    }
}

/// Rejects the call if `addr` is on the blacklist.
fn require_not_blacklisted(env: &Env, addr: &Address) -> Result<(), Error> {
    if is_blacklisted(env, addr) {
        Err(Error::Blacklisted)
    } else {
        Ok(())
    }
}

#[contract]
pub struct CarbonCreditToken;

#[contractimpl]
impl CarbonCreditToken {
    /// Initializes the contract with admin/super-admin and metadata.
    /// Can only be called once.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<(), Error> {
        if is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        set_initialized(&env);
        write_administrator(&env, &admin);
        // The initial admin is also the SuperAdmin.
        write_super_admin(&env, &admin);
        write_metadata(&env, name, symbol, decimals);
        write_total_supply(&env, 0);
        write_total_retired(&env, 0);

        Ok(())
    }

    // ── RBAC management (SuperAdmin only) ────────────────────────────────────

    /// Grants the Verifier role to `verifier`.
    /// Only the SuperAdmin may call this.
    pub fn add_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        grant_verifier(&env, &verifier);
        Ok(())
    }

    /// Revokes the Verifier role from `verifier`.
    /// Only the SuperAdmin may call this.
    pub fn remove_verifier(env: Env, verifier: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        revoke_verifier(&env, &verifier);
        Ok(())
    }

    /// Blacklists `target`.
    /// Only the SuperAdmin may call this.
    /// The SuperAdmin cannot blacklist themselves — they must transfer the role
    /// first via `transfer_super_admin`.
    pub fn blacklist(env: Env, target: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        if target == super_admin {
            return Err(Error::CannotBlacklistSelf);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        blacklist_address(&env, &target);
        Ok(())
    }

    /// Removes `target` from the blacklist.
    /// Only the SuperAdmin may call this.
    pub fn unblacklist(env: Env, target: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        unblacklist_address(&env, &target);
        Ok(())
    }

    /// Transfers the SuperAdmin role to `successor`.
    /// The successor must be a different address.
    /// Only the current SuperAdmin may call this.
    pub fn transfer_super_admin(env: Env, successor: Address) -> Result<(), Error> {
        let super_admin = read_super_admin(&env);
        super_admin.require_auth();

        if successor == super_admin {
            return Err(Error::InvalidSuccessor);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_super_admin(&env, &successor);
        Ok(())
    }

    // ── Token operations ──────────────────────────────────────────────────────

    /// Mints tokens to an address (Admin only).
    /// Used to credit verified donations.
    pub fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error> {
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &to)?;

        let admin = read_administrator(&env);
        admin.require_auth();

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        receive_balance(&env, to.clone(), amount);

        let new_supply = read_total_supply(&env) + amount;
        write_total_supply(&env, new_supply);

        MintEvent {
            to: to.clone(),
            amount,
        }
        .publish(&env);

        Ok(())
    }

    /// Transfers tokens between addresses.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;
        require_not_blacklisted(&env, &to)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;
        receive_balance(&env, to.clone(), amount);

        TransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount,
        }
        .publish(&env);

        Ok(())
    }

    /// Transfers tokens using allowance.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), Error> {
        spender.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &spender)?;
        require_not_blacklisted(&env, &from)?;
        require_not_blacklisted(&env, &to)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount)?;
        spend_balance(&env, from.clone(), amount)?;
        receive_balance(&env, to.clone(), amount);

        TransferEvent { from, to, amount }.publish(&env);

        Ok(())
    }

    /// Retires (burns) tokens to claim carbon offset.
    /// Emits a special RetirementEvent with timestamp.
    pub fn retire(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;

        if amount == 0 {
            return Err(Error::ZeroRetirementAmount);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        let new_retired = read_total_retired(&env) + amount;
        write_total_retired(&env, new_retired);

        let timestamp = env.ledger().timestamp();

        RetirementEvent {
            from: from.clone(),
            amount,
            timestamp,
        }
        .publish(&env);

        BurnEvent { from, amount }.publish(&env);

        Ok(())
    }

    /// Burns tokens (SEP-41 standard).
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);

        Ok(())
    }

    /// Burns tokens using allowance.
    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) -> Result<(), Error> {
        spender.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &spender)?;
        require_not_blacklisted(&env, &from)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount)?;
        spend_balance(&env, from.clone(), amount)?;

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);

        Ok(())
    }

    /// Approves spending by a spender (SEP-41).
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), Error> {
        from.require_auth();
        check_nonnegative_amount(amount)?;
        require_not_blacklisted(&env, &from)?;

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(
            &env,
            from.clone(),
            spender.clone(),
            amount,
            expiration_ledger,
        )?;

        ApproveEvent {
            from,
            spender,
            amount,
            expiration_ledger,
        }
        .publish(&env);

        Ok(())
    }

    // ── View functions ────────────────────────────────────────────────────────

    /// Returns `true` if `addr` holds the Verifier role.
    pub fn is_verifier(env: Env, addr: Address) -> bool {
        is_verifier(&env, &addr)
    }

    /// Returns `true` if `addr` is blacklisted.
    pub fn is_blacklisted(env: Env, addr: Address) -> bool {
        is_blacklisted(&env, &addr)
    }

    /// Returns the balance of an address.
    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_balance(&env, id)
    }

    /// Returns the allowance of a spender.
    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        read_allowance(&env, from, spender)
    }

    /// Returns the total supply of tokens in circulation.
    pub fn total_supply(env: Env) -> i128 {
        read_total_supply(&env)
    }

    /// Returns the total tokens retired (CO₂ offset claimed).
    pub fn total_retired(env: Env) -> i128 {
        read_total_retired(&env)
    }

    /// Returns the token name.
    pub fn name(env: Env) -> String {
        read_name(&env)
    }

    /// Returns the token symbol.
    pub fn symbol(env: Env) -> String {
        read_symbol(&env)
    }

    /// Returns the token decimals.
    pub fn decimals(env: Env) -> u32 {
        read_decimals(&env)
    }
}
