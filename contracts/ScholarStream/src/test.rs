//! Test suite for ScholarStream.
//!
//! Exactly 5 tests, as required:
//! 1. Happy path         - full end-to-end scholarship claim
//! 2. Edge case          - claim attempted without a verified credential fails
//! 3. State verification - storage correctly reflects state after a claim
//! 4. Duplicate claim    - a student cannot claim twice
//! 5. Unauthorized issuer - a non-university address cannot issue credentials
//!
//! NOTE ON TOKEN SETUP: this mocks a Stellar Asset Contract (the standard way
//! to represent USDC-like tokens on Soroban) using the SDK's test helper.
//! The helper name (`register_stellar_asset_contract_v2` below) has changed
//! across soroban-sdk releases — if your installed version differs, swap in
//! the equivalent helper from `soroban_sdk::testutils` for your SDK version.

use super::{ScholarStreamContract, ScholarStreamContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env};

/// Helper: sets up a fresh Env, a mock USDC-like token, and an initialized
/// ScholarStream contract. Returns the pieces each test needs.
fn setup() -> (
    Env,
    ScholarStreamContractClient<'static>,
    token::StellarAssetClient<'static>,
    token::Client<'static>,
    Address, // admin (NGO)
    Address, // university
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let university = Address::generate(&env);
    let token_admin = Address::generate(&env);

    // Deploy a mock Stellar Asset Contract to stand in for USDC.
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract_id.address());
    let token_client = token::Client::new(&env, &token_contract_id.address());

    let contract_id = env.register(ScholarStreamContract, ());
    let client = ScholarStreamContractClient::new(&env, &contract_id);

    let scholarship_amount: i128 = 500_0000000; // 500 USDC (7 decimals)
    client.initialize(
        &admin,
        &university,
        &token_contract_id.address(),
        &scholarship_amount,
    );

    // Fund the contract so it can pay out scholarships.
    token_admin_client.mint(&contract_id, &(scholarship_amount * 10));

    (env, client, token_admin_client, token_client, admin, university)
}

#[test]
fn test_happy_path_claim_scholarship() {
    let (env, client, _token_admin, token_client, _admin, university) = setup();
    let student = Address::generate(&env);

    // University issues Priya's enrollment credential.
    client.issue_credential(&university, &student, &true);

    // Priya claims her scholarship.
    client.claim_scholarship(&student);

    // Her wallet now holds the scholarship amount.
    assert_eq!(token_client.balance(&student), 500_0000000);
}

#[test]
fn test_claim_without_credential_fails() {
    let (env, client, ..) = setup();
    let student = Address::generate(&env);

    // No credential was ever issued for this student.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.claim_scholarship(&student);
    }));

    assert!(result.is_err(), "claim should fail without a verified credential");
}

#[test]
fn test_state_after_claim() {
    let (env, client, _token_admin, _token_client, _admin, university) = setup();
    let student = Address::generate(&env);

    client.issue_credential(&university, &student, &true);

    assert_eq!(client.is_verified(&student), true);
    assert_eq!(client.has_claimed(&student), false);

    client.claim_scholarship(&student);

    assert_eq!(client.is_verified(&student), true);
    assert_eq!(client.has_claimed(&student), true);
}

#[test]
fn test_duplicate_claim_fails() {
    let (env, client, _token_admin, _token_client, _admin, university) = setup();
    let student = Address::generate(&env);

    client.issue_credential(&university, &student, &true);
    client.claim_scholarship(&student);

    // Second claim attempt must fail.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.claim_scholarship(&student);
    }));

    assert!(result.is_err(), "a student must not be able to claim twice");
}

#[test]
fn test_unauthorized_university_cannot_issue_credential() {
    let (env, client, ..) = setup();
    let student = Address::generate(&env);
    let imposter = Address::generate(&env);

    // `imposter` is not the registered university for this contract.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.issue_credential(&imposter, &student, &true);
    }));

    assert!(
        result.is_err(),
        "only the registered university may issue credentials"
    );
}