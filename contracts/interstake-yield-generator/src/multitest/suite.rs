use anyhow::Result as AnyResult;
use cw_utils::Expiration;
use interstake_yield_generator_v02::contract as yield_generator_v02;
use interstake_yield_generator_v02::msg as msg_v02;
use interstake_yield_generator_v03::contract as yield_generator_v03;
use interstake_yield_generator_v03::msg as msg_v03;
use schemars::JsonSchema;
use serde::Serialize;

use std::fmt;

use cosmwasm_std::{
    Addr, AllDelegationsResponse, BlockInfo, Coin, Decimal, Delegation, StakingQuery, Uint128,
    Validator,
};
use cw_multi_test::{
    App, AppResponse, Contract, ContractWrapper, Executor, StakingInfo, StakingSudo, SudoMsg,
};

use crate::msg::PendingClaimResponse;
use crate::msg::{
    AllowedAddrResponse, ClaimsResponse, ConfigResponse, DelegateResponse, DelegatedResponse,
    ExecuteMsg, InstantiateMsg, LastPaymentBlockResponse, QueryMsg, RewardResponse,
    TotalDelegatedResponse, ValidatorsResponse,
};
use crate::state::{ClaimDetails, Config};

pub const TWENTY_EIGHT_DAYS: u64 = 3600 * 24 * 28;
pub const FOUR_DAYS: u64 = 3600 * 24 * 4;

pub fn contract_yield_generator<C>() -> Box<dyn Contract<C>>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
{
    let contract = ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    let contract = contract.with_migrate_empty(crate::contract::migrate);
    Box::new(contract)
}

pub fn contract_yield_generator_v03<C>() -> Box<dyn Contract<C>>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
{
    let contract = ContractWrapper::new_with_empty(
        yield_generator_v03::execute,
        yield_generator_v03::instantiate,
        yield_generator_v03::query,
    );
    let contract = contract.with_migrate_empty(yield_generator_v03::migrate);
    Box::new(contract)
}

pub fn contract_yield_generator_v02<C>() -> Box<dyn Contract<C>>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
{
    let contract = ContractWrapper::new_with_empty(
        yield_generator_v02::execute,
        yield_generator_v02::instantiate,
        yield_generator_v02::query,
    );
    Box::new(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    pub owner: String,
    pub treasury: String,
    pub restake_commission: Decimal,
    pub transfer_commission: Decimal,
    pub validator_commission: Decimal,
    pub number_of_validators: u32,
    pub funds: Vec<(Addr, Vec<Coin>)>,
    pub denom: String,
}

pub const VALIDATOR_1: &str = "validator1";
pub const VALIDATOR_2: &str = "validator2";

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            owner: "owner".to_owned(),
            restake_commission: Decimal::zero(),
            transfer_commission: Decimal::zero(),
            validator_commission: Decimal::percent(5),
            number_of_validators: 2,
            treasury: "treasury".to_owned(),
            funds: vec![],
            denom: "ujuno".to_owned(),
        }
    }

    pub fn with_multiple_validators(mut self, number_of_validators: u32) -> Self {
        self.number_of_validators = number_of_validators;
        self
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    pub fn with_multiple_funds(mut self, user_funds: &[(Addr, Vec<Coin>)]) -> Self {
        for userfunds in user_funds {
            self.funds.push(userfunds.clone());
        }
        self
    }

    pub fn with_restake_commission(mut self, commission: Decimal) -> Self {
        self.restake_commission = commission;
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let owner = Addr::unchecked(self.owner.clone());
        let treasury = Addr::unchecked(self.treasury.clone());
        let funds = self.funds;

        let mut app: App = App::default();

        let mut validators: Vec<Validator> = vec![];
        for number in 1..=self.number_of_validators {
            let validator = Validator {
                address: format!("validator{}", number),
                commission: self.validator_commission,
                max_commission: Decimal::percent(100),
                max_change_rate: Decimal::percent(1),
            };
            validators.push(validator);
        }

        let staking_info = StakingInfo {
            bonded_denom: "ujuno".to_string(),
            unbonding_time: 60,
            apr: Decimal::percent(80),
        };

        let block_info = app.block_info();
        // Use init_modules to setup some initial validator with a stake
        app.init_modules(|router, api, storage| -> AnyResult<()> {
            router.staking.setup(storage, staking_info).unwrap();

            validators.into_iter().for_each(|validator| {
                router
                    .staking
                    .add_validator(api, storage, &block_info, validator)
                    .unwrap();
            });

            funds.into_iter().for_each(|(address, coins)| {
                router.bank.init_balance(storage, &address, coins).unwrap()
            });

            Ok(())
        })
        .unwrap();

        let yield_generator_id = app.store_code(contract_yield_generator());
        let yield_generator_contract = app
            .instantiate_contract(
                yield_generator_id,
                owner.clone(),
                &InstantiateMsg {
                    owner: self.owner.clone(),
                    treasury: self.treasury.clone(),
                    staking_addr: VALIDATOR_1.to_owned(),
                    restake_commission: self.restake_commission,
                    transfer_commission: self.restake_commission,
                    denom: self.denom.clone(),
                    unbonding_period: Some(TWENTY_EIGHT_DAYS),
                    max_entries: Some(7),
                },
                &[],
                "yield_generator",
                None,
            )
            .unwrap();

        let yield_generator_v02_id = app.store_code(contract_yield_generator_v02());
        let yield_generator_v02_contract = app
            .instantiate_contract(
                yield_generator_v02_id,
                owner.clone(),
                &msg_v02::InstantiateMsg {
                    owner: self.owner.clone(),
                    staking_addr: VALIDATOR_1.to_owned(),
                    team_commision: Some(self.restake_commission),
                    denom: self.denom.clone(),
                    unbonding_period: Some(TWENTY_EIGHT_DAYS),
                },
                &[],
                "yield_generator_v02",
                Some(owner.to_string()),
            )
            .unwrap();

        let yield_generator_v03_id = app.store_code(contract_yield_generator_v03());
        let yield_generator_v03_contract = app
            .instantiate_contract(
                yield_generator_v03_id,
                owner.clone(),
                &msg_v03::InstantiateMsg {
                    owner: self.owner.clone(),
                    staking_addr: VALIDATOR_1.to_owned(),
                    denom: self.denom,
                    unbonding_period: Some(TWENTY_EIGHT_DAYS),
                    treasury: self.treasury.to_string(),
                    restake_commission: self.restake_commission,
                    transfer_commission: self.transfer_commission,
                },
                &[],
                "yield_generator_v03",
                Some(owner.to_string()),
            )
            .unwrap();
        Suite {
            app,
            owner,
            treasury,
            contract: yield_generator_contract,
            contract_code_id: yield_generator_id,
            contract_v02: yield_generator_v02_contract,
            contract_v03: yield_generator_v03_contract,
            contract_v02_code_id: yield_generator_v02_id,
            contract_v03_code_id: yield_generator_v03_id,
        }
    }
}

pub struct Suite {
    pub app: App,
    owner: Addr,
    treasury: Addr,
    pub contract: Addr,
    pub contract_code_id: u64,
    pub contract_v02: Addr,
    pub contract_v03: Addr,
    pub contract_v02_code_id: u64,
    pub contract_v03_code_id: u64,
}

pub fn validator_list(i: u32) -> Vec<(String, Decimal)> {
    let mut validators = vec![];
    //equally devide validators
    let weight = Decimal::from_ratio(1u128, (i) as u128);

    for i in 0..i {
        validators.push((format!("validator{}", i + 1), weight));
    }
    validators
}

pub fn two_false_validators() -> Vec<(String, Decimal)> {
    vec![
        (VALIDATOR_1.to_string(), Decimal::percent(25)),
        (VALIDATOR_2.to_string(), Decimal::percent(50)),
    ]
}
impl Suite {
    pub fn owner(&self) -> Addr {
        self.owner.clone()
    }

    pub fn treasury(&self) -> Addr {
        self.treasury.clone()
    }

    pub fn advance_height(&mut self, blocks: u64) {
        self.app.update_block(|block: &mut BlockInfo| {
            block.time = block.time.plus_seconds(5 * blocks);
            block.height += blocks;
        })
    }

    pub fn advance_time(&mut self, time: u64) {
        self.app.update_block(|block: &mut BlockInfo| {
            block.time = block.time.plus_seconds(time);
            block.height += time / 5;
        })
    }

    pub fn process_staking_queue(&mut self) -> AnyResult<AppResponse> {
        self.app
            .sudo(SudoMsg::Staking(StakingSudo::ProcessQueue {}))
    }

    pub fn update_config(
        &mut self,
        sender: &str,
        owner: impl Into<Option<String>>,
        treasury: impl Into<Option<String>>,
        restake_commission: impl Into<Option<Decimal>>,
        transfer_commission: impl Into<Option<Decimal>>,
        unbonding_period: impl Into<Option<u64>>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UpdateConfig {
                owner: owner.into(),
                treasury: treasury.into(),
                restake_commission: restake_commission.into(),
                transfer_commission: transfer_commission.into(),
                unbonding_period: unbonding_period.into(),
            },
            &[],
        )
    }

    pub fn update_validator_list(
        &mut self,
        sender: &str,
        new_validator_list: Vec<(String, Decimal)>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UpdateValidatorList { new_validator_list },
            &[],
        )
    }

    pub fn delegate(&mut self, sender: &str, amount: Coin) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Delegate {},
            &[amount],
        )
    }

    pub fn undelegate(&mut self, sender: &str, amount: Coin) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Undelegate { amount },
            &[],
        )
    }

    pub fn undelegate_all(&mut self, sender: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UndelegateAll {},
            &[],
        )
    }

    pub fn restake(&mut self, sender: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Restake {},
            &[],
        )
    }

    pub fn claim(&mut self, sender: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Claim {},
            &[],
        )
    }

    pub fn transfer(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: Uint128,
        commission_address: Option<String>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Transfer {
                recipient: recipient.into(),
                amount,
                commission_address,
            },
            &[],
        )
    }

    pub fn update_allowed_addr(
        &mut self,
        sender: &str,
        addr: &str,
        expires: u64,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UpdateAllowedAddr {
                address: addr.into(),
                expires,
            },
            &[],
        )
    }

    pub fn remove_allowed_addr(&mut self, sender: &str, addr: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::RemoveAllowedAddr {
                address: addr.into(),
            },
            &[],
        )
    }

    pub fn batch_unbond(&mut self, sender: &str) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::BatchUnbond {},
            &[],
        )
    }

    pub fn query_config(&self) -> AnyResult<Config> {
        let response: ConfigResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Config {})?;
        Ok(response.config)
    }

    pub fn query_validator_list(&self) -> AnyResult<Vec<(String, Decimal)>> {
        let response: ValidatorsResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::ValidatorList {})?;
        Ok(response.validators)
    }

    pub fn query_delegated(&self, sender: impl Into<String>) -> AnyResult<DelegateResponse> {
        let response: DelegatedResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Delegated {
                sender: sender.into(),
            },
        )?;
        Ok(response.delegated[0].clone())
    }

    pub fn query_total_delegated(&self) -> AnyResult<TotalDelegatedResponse> {
        let response: TotalDelegatedResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::TotalDelegated {})?;
        Ok(response)
    }

    pub fn query_reward(&self) -> AnyResult<Coin> {
        let response: RewardResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Reward {})?;
        Ok(response.rewards[0].clone())
    }

    pub fn query_last_payment_block(&self) -> AnyResult<u64> {
        let response: LastPaymentBlockResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::LastPaymentBlock {})?;
        Ok(response.last_payment_block)
    }

    pub fn query_pending_claims(&self, sender: impl Into<String>) -> AnyResult<Uint128> {
        let response: PendingClaimResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::PendingClaim {
                sender: sender.into(),
            },
        )?;
        Ok(response.amount)
    }

    pub fn query_claims(&self, sender: impl Into<String>) -> AnyResult<Vec<ClaimDetails>> {
        let response: ClaimsResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Claims {
                sender: sender.into(),
            },
        )?;
        Ok(response.claims)
    }

    pub fn query_all_delegations(&self) -> AnyResult<Vec<Delegation>> {
        let response: AllDelegationsResponse = self.app.wrap().query(
            &cosmwasm_std::QueryRequest::Staking(StakingQuery::AllDelegations {
                delegator: self.contract.to_string(),
            }),
        )?;
        Ok(response.delegations)
    }

    pub fn query_allowed_addr(&self, address: &str) -> AnyResult<Expiration> {
        let response: AllowedAddrResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::AllowedAddr {
                address: address.to_string(),
            },
        )?;
        Ok(response.expires)
    }

    pub fn migrate<T: Serialize>(
        &mut self,
        sender: &str,
        contract: Addr,
        code_id: u64,
        msg: &T,
    ) -> AnyResult<AppResponse> {
        self.contract = self.contract_v02.clone();
        self.app
            .migrate_contract(Addr::unchecked(sender), contract, msg, code_id)
    }

    pub fn query_contract_config(&self, contract: Addr) -> AnyResult<Config> {
        let response: ConfigResponse = self
            .app
            .wrap()
            .query_wasm_smart(contract, &QueryMsg::Config {})?;
        Ok(response.config)
    }
}
