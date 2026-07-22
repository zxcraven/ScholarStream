#![no_std]

//! ScholarStream
//!
//! A university issues an on-chain enrollment credential for a student.
//! Once that credential is verified, the student can claim their scholarship
//! directly — the contract checks the credential and disburses funds (USDC)
//! from its own balance instantly, with no manual approval chain.

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

/// Storage keys used by the contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// NGO admin address — the entity that funds and configures the contract.
    Admin,
    /// The single university address authorized to issue credentials.
    University,
    /// The token contract address used for disbursement (e.g. USDC on Stellar).
    Token,
    /// The fixed scholarship amount paid out per verified student.
    ScholarshipAmount,
    /// Per-student enrollment credential: student Address -> verified bool.
    Credential(Address),
    /// Per-student claim flag, used to prevent double-claiming.
    Claimed(Address),
}

#[contract]
pub struct ScholarStreamContract;

#[contractimpl]
impl ScholarStreamContract {
    /// Initialize the contract. Called once by the NGO admin.
    ///
    /// - `admin`: the NGO's address, must authorize this call.
    /// - `university`: the only address allowed to issue credentials.
    /// - `token`: the Stellar token contract address (e.g. USDC) used to pay students.
    /// - `amount`: the fixed scholarship amount disbursed per student.
    pub fn initialize(env: Env, admin: Address, university: Address, token: Address, amount: i128) {
        admin.require_auth();

        if amount <= 0 {
            panic!("scholarship amount must be positive");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::University, &university);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage()
            .instance()
            .set(&DataKey::ScholarshipAmount, &amount);
    }

    /// University issues (or revokes) a verifiable enrollment credential for a student.
    /// Only the single registered university address may call this — anyone else
    /// attempting to issue a credential is rejected.
    pub fn issue_credential(env: Env, university: Address, student: Address, verified: bool) {
        university.require_auth();

        let registered_university: Address = env
            .storage()
            .instance()
            .get(&DataKey::University)
            .expect("contract not initialized");

        if university != registered_university {
            panic!("unauthorized: only the registered university may issue credentials");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Credential(student), &verified);
    }

    /// Returns whether a student currently has a verified enrollment credential.
    pub fn is_verified(env: Env, student: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Credential(student))
            .unwrap_or(false)
    }

    /// Returns whether a student has already claimed their scholarship.
    pub fn has_claimed(env: Env, student: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(student))
            .unwrap_or(false)
    }

    /// Student claims their scholarship.
    ///
    /// Flow: verify the student authorized this call -> check their credential
    /// is verified -> check they haven't already claimed -> transfer the
    /// scholarship amount from the contract's token balance directly to the
    /// student's wallet -> mark them as claimed so it can't be repeated.
    pub fn claim_scholarship(env: Env, student: Address) {
        student.require_auth();

        let verified: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(student.clone()))
            .unwrap_or(false);

        if !verified {
            panic!("no verified enrollment credential found for this student");
        }

        let already_claimed: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Claimed(student.clone()))
            .unwrap_or(false);

        if already_claimed {
            panic!("scholarship already claimed by this student");
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("contract not initialized");
        let amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::ScholarshipAmount)
            .expect("contract not initialized");

        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&env.current_contract_address(), &student, &amount);

        env.storage()
            .persistent()
            .set(&DataKey::Claimed(student), &true);
    }
}

#[cfg(test)]
mod test;