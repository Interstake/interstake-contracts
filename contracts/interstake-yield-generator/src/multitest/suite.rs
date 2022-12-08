use anyhow::Result as AnyResult;
use schemars::JsonSchema;
use std::fmt;

use cosmwasm_std::{
    Addr, AllDelegationsResponse, BlockInfo, Coin, Decimal, Delegation, StakingQuery, Uint128,
    Validator,
};
use cw_multi_test::{
    App, AppResponse, Contract, ContractWrapper, Executor, StakingInfo, StakingSudo, SudoMsg,
};

use crate::msg::{
    ClaimsResponse, ConfigResponse, DelegateResponse, DelegatedResponse, ExecuteMsg,
    InstantiateMsg, LastPaymentBlockResponse, QueryMsg, RewardResponse, TotalDelegatedResponse,
    ValidatorsResponse,
};
use crate::state::{ClaimDetails, Config};

pub const TWENTY_EIGHT_DAYS: u64 = 3600 * 24 * 28;

pub fn contract_yield_generator<C>() -> Box<dyn Contract<C>>
where
    C: Clone + fmt::Debug + PartialEq + JsonSchema + 'static,
{
    let contract = ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

#[derive(Debug)]
pub struct SuiteBuilder {
    pub owner: String,
    pub team_commision: Decimal,
    pub validator_commission: Decimal,
    pub funds: Vec<(Addr, Vec<Coin>)>,
    pub denom: String,
}

pub const VALIDATOR_1: &str = "validator1";
pub const VALIDATOR_2: &str = "validator2";

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            owner: "owner".to_owned(),
            team_commision: Decimal::zero(),
            validator_commission: Decimal::percent(5),
            funds: vec![],
            denom: "ujuno".to_owned(),
        }
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

    #[track_caller]
    pub fn build(self) -> Suite {
        let owner = Addr::unchecked(self.owner.clone());
        let funds = self.funds;

        let mut app: App = App::default();

        let valoper1 = Validator {
            address: VALIDATOR_1.to_owned(),
            commission: self.validator_commission,
            max_commission: Decimal::percent(100),
            max_change_rate: Decimal::percent(1),
        };

        let valoper2 = Validator {
            address: VALIDATOR_2.to_owned(),
            commission: self.validator_commission,
            max_commission: Decimal::percent(100),
            max_change_rate: Decimal::percent(1),
        };

        let staking_info = StakingInfo {
            bonded_denom: "ujuno".to_string(),
            unbonding_time: 60,
            apr: Decimal::percent(80),
        };

        let block_info = app.block_info();
        // Use init_modules to setup some initial validator with a stake
        app.init_modules(|router, api, storage| -> AnyResult<()> {
            router.staking.setup(storage, staking_info).unwrap();

            router
                .staking
                .add_validator(api, storage, &block_info, valoper1)
                .unwrap();

            // add second validator
            router
                .staking
                .add_validator(api, storage, &block_info, valoper2)
                .unwrap();

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
                    staking_addr: VALIDATOR_1.to_owned(),
                    team_commision: self.team_commision,
                    denom: self.denom,
                    unbonding_period: Some(TWENTY_EIGHT_DAYS),
                },
                &[],
                "yield_generator",
                None,
            )
            .unwrap();

        Suite {
            app,
            owner,
            contract: yield_generator_contract,
        }
    }
}

pub struct Suite {
    pub app: App,
    owner: Addr,
    contract: Addr,
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
        team_commision: impl Into<Option<Decimal>>,
        unbonding_period: impl Into<Option<u64>>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UpdateConfig {
                owner: owner.into(),
                team_commision: team_commision.into(),
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
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::Transfer {
                recipient: recipient.into(),
                amount,
            },
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
}
