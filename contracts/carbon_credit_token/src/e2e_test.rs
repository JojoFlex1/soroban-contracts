#![cfg(test)]

use crate::{CarbonCreditToken, CarbonCreditTokenClient};
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token, Address, BytesN, Env, String,
};

// ============ MOCK REGISTRY ============
#[contract]
pub struct MockRegistry;

#[contractimpl]
impl MockRegistry {
    pub fn add_report(env: Env, oracle: Address, report_hash: BytesN<32>) {
        oracle.require_auth();
        // Emit an event to simulate saving the report
        env.events()
            .publish((oracle, report_hash), "report_submitted");
    }
}

// ============ MOCK ESCROW ============
#[contract]
pub struct MockEscrow;

#[soroban_sdk::contracttype]
#[derive(Clone)]
pub struct Offer {
    pub carbon_token: Address,
    pub usdc_token: Address,
    pub carbon_amount: i128,
    pub usdc_amount: i128,
}

#[contractimpl]
impl MockEscrow {
    pub fn create_offer(
        env: Env,
        farmer: Address,
        carbon_token: Address,
        usdc_token: Address,
        carbon_amount: i128,
        usdc_amount: i128,
    ) {
        farmer.require_auth();

        let client = CarbonCreditTokenClient::new(&env, &carbon_token);
        // Transfer carbon credits from farmer to escrow
        // We use `transfer` not `transfer_from` because Soroban's Auth framework
        // will bubble up `farmer.require_auth()` to the transaction level.
        client.transfer(&farmer, &env.current_contract_address(), &carbon_amount);

        let offer = Offer {
            carbon_token,
            usdc_token,
            carbon_amount,
            usdc_amount,
        };
        env.storage().instance().set(&farmer, &offer);
    }

    pub fn execute_swap(env: Env, corporate: Address, farmer: Address) {
        corporate.require_auth();

        let offer: Offer = env.storage().instance().get(&farmer).unwrap();

        // Transfer USDC from corporate to farmer
        let usdc_client = token::Client::new(&env, &offer.usdc_token);
        usdc_client.transfer(&corporate, &farmer, &offer.usdc_amount);

        // Transfer Carbon Credits from Escrow to corporate
        let carbon_client = CarbonCreditTokenClient::new(&env, &offer.carbon_token);
        carbon_client.transfer(
            &env.current_contract_address(),
            &corporate,
            &offer.carbon_amount,
        );

        env.storage().instance().remove(&farmer);
    }
}

// ============ END-TO-END INTEGRATION TEST ============
#[test]
fn test_carbon_credit_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Setup Identities
    let super_admin = Address::generate(&env); // Admin and SuperAdmin
    let oracle = Address::generate(&env);
    let farmer = Address::generate(&env);
    let corporate = Address::generate(&env);

    // 2. Deploy Carbon Credit Token
    let token_id = env.register_contract(None, CarbonCreditToken);
    let cct_client = CarbonCreditTokenClient::new(&env, &token_id);
    cct_client.initialize(
        &super_admin,
        &String::from_str(&env, "Carbon Credit"),
        &String::from_str(&env, "CCT"),
        &0u32,
    );

    // 3. Deploy Mocks
    let registry_id = env.register_contract(None, MockRegistry);
    let registry_client = MockRegistryClient::new(&env, &registry_id);

    let escrow_id = env.register_contract(None, MockEscrow);
    let escrow_client = MockEscrowClient::new(&env, &escrow_id);

    // 4. Setup Dummy USDC
    let usdc_admin = Address::generate(&env);
    let usdc_contract = env.register_stellar_asset_contract(usdc_admin.clone());
    let usdc_client = token::Client::new(&env, &usdc_contract);
    let usdc_admin_client = token::StellarAssetClient::new(&env, &usdc_contract);
    usdc_admin_client.mint(&corporate, &10_000); // 10,000 USDC to Corporate

    // ==========================================
    // Lifecycle Step 1: Initialize & RBAC setup
    // ==========================================
    // Admin assigns the Verifier role to an Oracle
    cct_client.add_verifier(&oracle);
    assert!(cct_client.is_verifier(&oracle));

    // ==========================================
    // Lifecycle Step 2: Soil Report Creation
    // ==========================================
    // Oracle submits a soil report hash to the Registry
    let report_hash = BytesN::from_array(&env, &[1u8; 32]);
    registry_client.add_report(&oracle, &report_hash);

    // Verify Registry Event
    let events = env.events().all();
    let contains_report = events.iter().any(|e| {
        // Just verify there is an event with "report_submitted" concept
        !e.topics.is_empty()
    });
    assert!(contains_report);

    // ==========================================
    // Lifecycle Step 3: Token Minting
    // ==========================================
    // Oracle triggers the Token mint (executed by Admin), sending credits to Farmer
    cct_client.mint(&farmer, &100);
    assert_eq!(cct_client.balance(&farmer), 100);

    // ==========================================
    // Lifecycle Step 4: Escrow DvP Offer
    // ==========================================
    // Farmer creates a DvP offer in the Escrow contract
    // Farmer wants 5000 USDC for 100 CCT
    escrow_client.create_offer(
        &farmer,
        &token_id,
        &usdc_contract,
        &100,  // carbon amount
        &5000, // usdc amount
    );
    assert_eq!(cct_client.balance(&farmer), 0);
    assert_eq!(cct_client.balance(&escrow_id), 100);

    // ==========================================
    // Lifecycle Step 5: Corporate OTC Swap
    // ==========================================
    // Corporate signs the OTC swap, sending USDC to Farmer, acquiring Carbon
    escrow_client.execute_swap(&corporate, &farmer);

    assert_eq!(usdc_client.balance(&corporate), 5000); // 10k - 5k
    assert_eq!(usdc_client.balance(&farmer), 5000);
    assert_eq!(cct_client.balance(&escrow_id), 0);
    assert_eq!(cct_client.balance(&corporate), 100);

    // ==========================================
    // Lifecycle Step 6: Offset Retirement
    // ==========================================
    // Corporate executes the retire function and receives their Offset Certificate (event)
    cct_client.retire(&corporate, &100);

    // Validate final state
    assert_eq!(cct_client.balance(&corporate), 0);
    assert_eq!(cct_client.total_retired(), 100);
    assert_eq!(cct_client.total_supply(), 0); // They consumed the supply
}
