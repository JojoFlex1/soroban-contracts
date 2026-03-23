use soroban_sdk::{Env, testutils::{Address as _, BytesN as _}, Address, BytesN};
use crate::{EscrowContract, Offer};

#[test]
fn test_fill_offer() {
    let env = Env::default();
    let buyer = Address::random(&env);
    let seller = Address::random(&env);
    let usdc_token = Address::random(&env);
    let carbon_token = Address::random(&env);
    let offer = Offer {
        offer_id: 1,
        buyer: buyer.clone(),
        seller: seller.clone(),
        usdc_amount: 1000,
        carbon_amount: 10,
        usdc_token: usdc_token.clone(),
        carbon_token: carbon_token.clone(),
    };
    let contract = EscrowContract {};
    contract.fill_offer(env.clone(), offer);
    // Add assertions for balances here as needed
}
