#![no_std]

mod admin;
mod allowance;
mod balance;
mod events;
mod metadata;
mod rbac;
mod storage;
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::admin::{read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::events::{ApproveEvent, BurnEvent, MintEvent, RetirementEvent, TransferEvent};
use crate::metadata::{read_decimals, read_name, read_symbol, write_metadata};
use crate::rbac::require_verifier;
use crate::storage::{
    is_initialized, read_total_retired, read_total_supply, set_initialized, write_rbac_contract,
    write_total_retired, write_total_supply, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount);
    }
}

#[contract]
pub struct CarbonCreditToken;

#[contractimpl]
impl CarbonCreditToken {
    /// Initializes the contract with admin, RBAC contract address, and token metadata.
    ///
    /// `rbac_contract` is the address of the deployed RBAC contract that will be
    /// queried on every `mint` call to verify the `Verifier` role.
    ///
    /// Can only be called once.
    pub fn initialize(
        env: Env,
        admin: Address,
        rbac_contract: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) {
        if is_initialized(&env) {
            panic!("contract already initialized");
        }

        set_initialized(&env);
        write_administrator(&env, &admin);
        write_rbac_contract(&env, &rbac_contract);
        write_metadata(&env, name, symbol, decimals);
        write_total_supply(&env, 0);
        write_total_retired(&env, 0);
    }

    /// Mints tokens to `to` (Verifier role required).
    ///
    /// The calling address must:
    ///   1. Sign the transaction (`require_auth`).
    ///   2. Hold the `"Verifier"` role in the registered RBAC contract.
    ///
    /// This replaces the previous monolithic-admin gate with a delegated,
    /// RBAC-driven authority model, allowing multiple vetted agricultural
    /// verifiers to issue credits independently without sharing a single
    /// admin key.
    pub fn mint(env: Env, verifier: Address, to: Address, amount: i128) {
        check_nonnegative_amount(amount);

        // Replaces: admin.require_auth()
        // The verifier must be authenticated AND carry the Verifier role.
        require_verifier(&env, &verifier);

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
    }

    /// Transfers tokens between addresses.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);

        TransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount,
        }
        .publish(&env);
    }

    /// Transfers tokens using allowance.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount);
        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);

        TransferEvent { from, to, amount }.publish(&env);
    }

    /// Retires (burns) tokens to claim a carbon offset.
    /// Emits a `RetirementEvent` with the ledger timestamp.
    pub fn retire(env: Env, from: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        if amount == 0 {
            panic!("retirement amount must be greater than zero");
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount);

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
    }

    /// Burns tokens (SEP-41 standard).
    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        check_nonnegative_amount(amount);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_balance(&env, from.clone(), amount);

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);
    }

    /// Burns tokens using allowance.
    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        check_nonnegative_amount(amount);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        spend_allowance(&env, from.clone(), spender, amount);
        spend_balance(&env, from.clone(), amount);

        let new_supply = read_total_supply(&env) - amount;
        write_total_supply(&env, new_supply);

        BurnEvent { from, amount }.publish(&env);
    }

    /// Approves spending by a spender (SEP-41).
    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();
        check_nonnegative_amount(amount);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_allowance(&env, from.clone(), spender.clone(), amount, expiration_ledger);

        ApproveEvent {
            from,
            spender,
            amount,
            expiration_ledger,
        }
        .publish(&env);
    }

    // ============ VIEW FUNCTIONS ============

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

    /// Returns the address of the RBAC contract used for role verification.
    pub fn rbac_contract(env: Env) -> Address {
        crate::storage::read_rbac_contract(&env)
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

    /// Returns the admin address (retained for non-minting governance).
    pub fn admin(env: Env) -> Address {
        read_administrator(&env)
    }
}