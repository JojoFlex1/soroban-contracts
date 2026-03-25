#![no_std]
use soroban_sdk::{contractimpl, contracttype, Env, Address, Symbol, Vec, require_auth};
use carbon_credit_token::Client as CarbonCreditTokenClient;

pub struct EscrowContract;

#[contracttype]
pub struct Offer {
    pub offer_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub usdc_amount: i128,
    pub carbon_amount: i128,
    pub usdc_token: Address,
    pub carbon_token: Address,
}

#[contractimpl]
impl EscrowContract {
    pub fn fill_offer(env: Env, offer: Offer) {
        // Authenticate buyer
        require_auth(&env, &offer.buyer);

        // Pull USDC from buyer to this contract (escrow)
        let usdc_client = carbon_credit_token::Client::new(&env, &offer.usdc_token);
        usdc_client.xfer_from(&offer.buyer, &env.current_contract_address(), &offer.usdc_amount);

        // Transfer Carbon Credits from escrow to buyer
        let carbon_client = carbon_credit_token::Client::new(&env, &offer.carbon_token);
        carbon_client.xfer_from(&env.current_contract_address(), &offer.buyer, &offer.carbon_amount);

        // Transfer USDC from escrow to seller
        usdc_client.xfer_from(&env.current_contract_address(), &offer.seller, &offer.usdc_amount);
    }
}
