#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Benefactor,   // Address that deposited funds
    Heir,         // Address that will receive funds
    Token,        // XLM / any SEP-41 token contract
    Amount,       // Total locked amount (i128)
    UnlockTime,   // Unix timestamp (u64) when heir can withdraw
    Claimed,      // bool – has the heir already withdrawn?
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct InheritanceLock;

#[contractimpl]
impl InheritanceLock {
    // ─── deposit ─────────────────────────────────────────────────────────
    /// Called once by the benefactor to lock funds.
    ///
    /// * `benefactor`   – wallet funding the inheritance
    /// * `heir`         – wallet that will receive the funds
    /// * `token`        – SEP-41 token contract (use Stellar's wrapped XLM address)
    /// * `amount`       – amount of tokens to lock (in base units / stroops)
    /// * `unlock_time`  – Unix timestamp (seconds) after which the heir may withdraw
    pub fn deposit(
        env: Env,
        benefactor: Address,
        heir: Address,
        token: Address,
        amount: i128,
        unlock_time: u64,
    ) {
        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Benefactor) {
            panic!("contract already initialised");
        }

        // Validate inputs
        if amount <= 0 {
            panic!("amount must be positive");
        }
        if unlock_time <= env.ledger().timestamp() {
            panic!("unlock_time must be in the future");
        }

        // Require benefactor signature
        benefactor.require_auth();

        // Pull tokens from benefactor into this contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&benefactor, &env.current_contract_address(), &amount);

        // Persist state
        env.storage().instance().set(&DataKey::Benefactor, &benefactor);
        env.storage().instance().set(&DataKey::Heir, &heir);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::UnlockTime, &unlock_time);
        env.storage().instance().set(&DataKey::Claimed, &false);

        env.events().publish(
            (Symbol::new(&env, "deposited"),),
            (benefactor, heir, amount, unlock_time),
        );
    }

    // ─── claim ────────────────────────────────────────────────────────────
    /// Called by the heir once the unlock time has passed.
    pub fn claim(env: Env) {
        // Load state
        let heir: Address = env
            .storage()
            .instance()
            .get(&DataKey::Heir)
            .expect("not initialised");
        let claimed: bool = env
            .storage()
            .instance()
            .get(&DataKey::Claimed)
            .unwrap_or(false);
        let unlock_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UnlockTime)
            .expect("not initialised");
        let amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Amount)
            .expect("not initialised");
        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("not initialised");

        // Guard: only the heir may call
        heir.require_auth();

        // Guard: must not have been claimed already
        if claimed {
            panic!("already claimed");
        }

        // Guard: time lock must have expired
        let now = env.ledger().timestamp();
        if now < unlock_time {
            panic!("funds still locked");
        }

        // Mark as claimed before transfer (re-entrancy safety)
        env.storage().instance().set(&DataKey::Claimed, &true);

        // Transfer tokens to heir
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &heir, &amount);

        env.events().publish(
            (Symbol::new(&env, "claimed"),),
            (heir, amount),
        );
    }

    // ─── revoke ───────────────────────────────────────────────────────────
    /// Emergency: benefactor can revoke the deposit before the heir claims it.
    /// Useful if the heir address was set incorrectly.
    pub fn revoke(env: Env) {
        let benefactor: Address = env
            .storage()
            .instance()
            .get(&DataKey::Benefactor)
            .expect("not initialised");
        let claimed: bool = env
            .storage()
            .instance()
            .get(&DataKey::Claimed)
            .unwrap_or(false);
        let amount: i128 = env
            .storage()
            .instance()
            .get(&DataKey::Amount)
            .expect("not initialised");
        let token: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .expect("not initialised");

        benefactor.require_auth();

        if claimed {
            panic!("already claimed by heir, cannot revoke");
        }

        // Mark claimed to prevent double-spend
        env.storage().instance().set(&DataKey::Claimed, &true);

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &benefactor, &amount);

        env.events().publish(
            (Symbol::new(&env, "revoked"),),
            (benefactor, amount),
        );
    }

    // ─── View helpers ─────────────────────────────────────────────────────

    pub fn get_unlock_time(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::UnlockTime)
            .expect("not initialised")
    }

    pub fn get_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::Amount)
            .expect("not initialised")
    }

    pub fn is_claimed(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Claimed)
            .unwrap_or(false)
    }

    pub fn time_remaining(env: Env) -> i64 {
        let unlock_time: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UnlockTime)
            .expect("not initialised");
        let now = env.ledger().timestamp();
        (unlock_time as i64) - (now as i64)
    }
}