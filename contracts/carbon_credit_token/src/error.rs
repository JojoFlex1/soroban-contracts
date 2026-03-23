use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The provided amount is negative.
    NegativeAmount = 1,
    /// The account does not have enough balance.
    InsufficientBalance = 2,
    /// The spender does not have enough allowance.
    InsufficientAllowance = 3,
    /// The contract has already been initialized.
    AlreadyInitialized = 4,
    /// The retirement amount must be greater than zero.
    ZeroRetirementAmount = 5,
    /// The allowance expiration ledger is in the past while amount > 0.
    InvalidExpirationLedger = 6,
}
