#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fund {
    pub name: String,
    pub creator: Address,
    pub total_contributions: i128,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Fund,
    Contribution(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FundError {
    FundAlreadyExists = 1,
    FundNotFound = 2,
    EmptyName = 3,
    InvalidAmount = 4,
    Overflow = 5,
}

#[contract]
pub struct ClassFund;

#[contractimpl]
impl ClassFund {
    pub fn create_fund(env: Env, creator: Address, name: String) -> Result<(), FundError> {
        creator.require_auth();

        if env.storage().persistent().has(&DataKey::Fund) {
            return Err(FundError::FundAlreadyExists);
        }
        if name.is_empty() {
            return Err(FundError::EmptyName);
        }

        let fund = Fund {
            name,
            creator,
            total_contributions: 0,
        };
        env.storage().persistent().set(&DataKey::Fund, &fund);

        Ok(())
    }

    /// Records a declared amount; this does not transfer or verify any tokens.
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), FundError> {
        let mut fund = read_fund(&env)?;
        contributor.require_auth();

        if amount <= 0 {
            return Err(FundError::InvalidAmount);
        }

        let key = DataKey::Contribution(contributor);
        let previous: i128 = env.storage().persistent().get(&key).unwrap_or(0);

        // Calculate both values before making either storage write.
        let new_total = fund
            .total_contributions
            .checked_add(amount)
            .ok_or(FundError::Overflow)?;
        let new_contribution = previous.checked_add(amount).ok_or(FundError::Overflow)?;

        fund.total_contributions = new_total;
        env.storage().persistent().set(&DataKey::Fund, &fund);
        env.storage().persistent().set(&key, &new_contribution);

        Ok(())
    }

    pub fn get_fund(env: Env) -> Result<Fund, FundError> {
        read_fund(&env)
    }

    pub fn get_contribution(env: Env, contributor: Address) -> Result<i128, FundError> {
        read_fund(&env)?;

        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0))
    }
}

fn read_fund(env: &Env) -> Result<Fund, FundError> {
    env.storage()
        .persistent()
        .get(&DataKey::Fund)
        .ok_or(FundError::FundNotFound)
}
