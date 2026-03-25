use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NegativeAmount = 2,
    Blacklisted = 3,
    CannotBlacklistSelf = 4,
    InvalidSuccessor = 5,
    ZeroRetirementAmount = 6,
    InsufficientBalance = 7,
    InsufficientAllowance = 8,
    InvalidExpirationLedger = 9,
}