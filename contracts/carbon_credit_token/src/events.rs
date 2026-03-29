use soroban_sdk::{contracttype, symbol_short, Address, Env};

#[derive(Clone, Debug)]
#[contracttype]
pub struct MintEvent {
    pub to: Address,
    pub amount: i128,
}

impl MintEvent {
    pub fn publish(self, env: &Env) {
        env.events()
            .publish((symbol_short!("mint"), self.to), self.amount);
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

impl TransferEvent {
    pub fn publish(self, env: &Env) {
        env.events()
            .publish((symbol_short!("transfer"), self.from, self.to), self.amount);
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct BurnEvent {
    pub from: Address,
    pub amount: i128,
}

impl BurnEvent {
    pub fn publish(self, env: &Env) {
        env.events()
            .publish((symbol_short!("burn"), self.from), self.amount);
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct ApproveEvent {
    pub from: Address,
    pub spender: Address,
    pub amount: i128,
    pub expiration_ledger: u32,
}

impl ApproveEvent {
    pub fn publish(self, env: &Env) {
        env.events().publish(
            (symbol_short!("approve"), self.from, self.spender),
            (self.amount, self.expiration_ledger),
        );
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct RetirementData {
    pub amount: i128,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct RetirementEvent {
    pub from: Address,
    pub amount: i128,
    pub timestamp: u64,
}

impl RetirementEvent {
    pub fn publish(self, env: &Env) {
        let data = RetirementData {
            amount: self.amount,
            timestamp: self.timestamp,
        };
        env.events()
            .publish((symbol_short!("retire"), self.from), data);
    }
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct CertificateGeneratedEvent {
    pub certificate_id: u64,
    pub corporate: Address,
    pub amount: i128,
    pub timestamp: u64,
}

impl CertificateGeneratedEvent {
    pub fn publish(self, env: &Env) {
        env.events().publish(
            (symbol_short!("cert"), self.corporate, self.certificate_id),
            (self.amount, self.timestamp),
        );
    }
}

