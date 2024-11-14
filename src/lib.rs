use near_sdk::json_types::U128;
use near_sdk::{env, near, require, store::LookupMap, AccountId, Gas, NearToken, Promise};
use near_sdk::{serde_json, GasWeight, PanicOnDefault, PromiseError};

// Define the contract structure
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    mpc_contract_id: AccountId,
    available_deposits: LookupMap<AccountId, NearToken>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(mpc_contract_id: AccountId) -> Self {
        Self {
            mpc_contract_id: mpc_contract_id,
            available_deposits: LookupMap::new(b"d"),
        }
    }

    // Non-payable
    pub fn sign(&mut self, sign_args: Vec<u8>, deposit: NearToken) -> Promise {
        require!(
            deposit.as_yoctonear() > 0,
            "Deposit allocated for MPC can't be zero"
        );

        require!(
            env::prepaid_gas() >= Gas::from_tgas(260),
            "Minimal prepaid gas is 260TGas as fewer amount won't be allowed by MPC anyway"
        );

        let predecessor = env::predecessor_account_id();

        let available_deposit = self
            .available_deposits
            .get(&predecessor)
            .expect("No available deposit");

        require!(
            available_deposit >= &deposit,
            "You don't have enough deposit to make this call"
        );

        // reduce deposits
        let updated_deposit =
            NearToken::from_yoctonear(available_deposit.as_yoctonear() - deposit.as_yoctonear());
        self.available_deposits
            .insert(predecessor.clone(), updated_deposit);

        let callback_promise = Contract::ext(env::current_account_id())
            .with_static_gas(Gas::from_tgas(5))
            .after_sign(predecessor.clone(), deposit);

        let sign_promise = Promise::new(self.mpc_contract_id.clone()).function_call_weight(
            "sign".to_string(),
            sign_args,
            deposit,
            Gas::from_tgas(250),
            GasWeight(1),
        );

        sign_promise.then(callback_promise)
    }

    pub fn available_deposit(&self, account_id: AccountId) -> U128 {
        let zero_deposit = NearToken::from_yoctonear(0);
        let balance = self
            .available_deposits
            .get(&account_id)
            .unwrap_or(&zero_deposit);

        U128(balance.as_yoctonear())
    }

    #[payable]
    pub fn deposit(&mut self) {
        let deposit = env::attached_deposit();

        require!(
            deposit.as_yoctonear() > 0,
            "Deposited amount must be greater than zero"
        );

        let predecessor = env::predecessor_account_id();

        let zero_deposit = NearToken::from_yoctonear(0);
        let current_deposit = self
            .available_deposits
            .get(&predecessor)
            .unwrap_or(&zero_deposit);

        let new_deposit =
            NearToken::from_yoctonear(current_deposit.as_yoctonear() + deposit.as_yoctonear());

        self.available_deposits.insert(predecessor, new_deposit);
    }

    pub fn withdraw(&mut self, amount: NearToken) {
        require!(
            amount.as_yoctonear() > 0,
            "Amount to withdraw must be bigger than zero"
        );
        let predecessor = env::predecessor_account_id();

        let zero_deposit = NearToken::from_yoctonear(0);
        let available_deposit = self
            .available_deposits
            .get(&predecessor)
            .unwrap_or(&zero_deposit);

        require!(
            &amount <= available_deposit,
            "Unsufficient available deposit to withdraw this amount"
        );

        let new_deposit =
            NearToken::from_yoctonear(available_deposit.as_yoctonear() - amount.as_yoctonear());
        self.available_deposits
            .insert(predecessor.clone(), new_deposit);

        Promise::new(predecessor).transfer(amount);
    }

    #[private]
    pub fn after_sign(
        &mut self,
        predecessor: AccountId,
        used_deposit: NearToken,
        #[callback_result] cb_result: Result<serde_json::Value, PromiseError>,
    ) -> serde_json::Value {
        // https://docs.rs/near-sdk/latest/near_sdk/env/fn.promise_results_count.html
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

        match cb_result {
            Ok(json) => json,
            Err(_) => {
                env::log_str(
                    format!(
                        "Signature wasn't generated, returning deposit {} yocto to {}",
                        used_deposit.as_yoctonear(),
                        predecessor
                    )
                    .as_str(),
                );

                let zero_deposit = NearToken::from_yoctonear(0);
                let current_deposit = self
                    .available_deposits
                    .get(&predecessor)
                    .unwrap_or(&zero_deposit);

                env::log_str(
                    format!("Current deposit {} yocto ", current_deposit.as_yoctonear(),).as_str(),
                );

                let new_deposit = NearToken::from_yoctonear(
                    current_deposit.as_yoctonear() + used_deposit.as_yoctonear(),
                );
                self.available_deposits.insert(predecessor, new_deposit);

                serde_json::Value::Null
            }
        }
    }
}
