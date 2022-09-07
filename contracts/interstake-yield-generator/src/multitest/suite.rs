use anyhow::Result as AnyResult;
use schemars::JsonSchema;
use std::fmt;

use cosmwasm_std::{coin, Addr, BlockInfo, Coin, Decimal};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

use crate::msg::{DelegateResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TotalDelegatedResponse};
use crate::state::{ClaimDetails, Config, TeamCommision};

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
    pub staking_addr: String,
    pub team_commision: Option<Decimal>,
    pub funds: Vec<(Addr, Vec<Coin>)>,
    pub denom: String,
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            owner: "owner".to_owned(),
            staking_addr: "staking".to_owned(),
            team_commision: None,
            funds: vec![],
            denom: "juno".to_owned(),
        }
    }

    /// Sets initial amount of distributable tokens on address
    pub fn with_funds(mut self, addr: &str, funds: &[Coin]) -> Self {
        self.funds.push((Addr::unchecked(addr), funds.into()));
        self
    }

    #[track_caller]
    pub fn build(self) -> Suite {
        let owner = Addr::unchecked(self.owner.clone());

        let funds = self.funds;
        let mut app: App = AppBuilder::new().build(|router, _, storage| {
            for (addr, coin) in funds {
                router.bank.init_balance(storage, &addr, coin).unwrap();
            }
            // FIXME: Dirty hack - prepare some tokens for contract to send back in undelegate scenarios
            // Proper solutions needs to be supplied in multi-test staking module
            // https://github.com/CosmWasm/cw-plus/pull/782
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("contract0"),
                    vec![coin(1000, "juno")],
                )
                .unwrap();
        });

        let yield_generator_id = app.store_code(contract_yield_generator());
        let yield_generator_contract = app
            .instantiate_contract(
                yield_generator_id,
                owner.clone(),
                &InstantiateMsg {
                    owner: self.owner.clone(),
                    staking_addr: self.staking_addr.to_string(),
                    team_commision: self.team_commision,
                    denom: self.denom,
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
            staking: self.staking_addr,
        }
    }
}

pub struct Suite {
    pub app: App,
    owner: Addr,
    contract: Addr,
    staking: String,
}

impl Suite {
    pub fn owner(&self) -> Addr {
        self.owner.clone()
    }

    pub fn staking(&self) -> String {
        self.staking.clone()
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

    pub fn update_config(
        &mut self,
        sender: &str,
        owner: impl Into<Option<String>>,
        staking_addr: impl Into<Option<String>>,
        team_commision: impl Into<Option<TeamCommision>>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            Addr::unchecked(sender),
            self.contract.clone(),
            &ExecuteMsg::UpdateConfig {
                owner: owner.into(),
                staking_addr: staking_addr.into(),
                team_commision: team_commision.into(),
            },
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

    pub fn query_config(&self) -> AnyResult<Config> {
        let response: Config = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Config {})?;
        Ok(response)
    }

    pub fn query_delegated(&self, sender: impl Into<String>) -> AnyResult<DelegateResponse> {
        let response: DelegateResponse = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Delegated {
                sender: sender.into(),
            },
        )?;
        Ok(response)
    }

    pub fn query_total_delegated(&self) -> AnyResult<TotalDelegatedResponse> {
        let response: TotalDelegatedResponse = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::TotalDelegated {})?;
        Ok(response)
    }

    pub fn query_reward(&self) -> AnyResult<Coin> {
        let response: Coin = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::Reward {})?;
        Ok(response)
    }

    pub fn query_last_payment_block(&self) -> AnyResult<u64> {
        let response: u64 = self
            .app
            .wrap()
            .query_wasm_smart(self.contract.clone(), &QueryMsg::LastPaymentBlock {})?;
        Ok(response)
    }

    pub fn query_claims(&self, sender: impl Into<String>) -> AnyResult<Vec<ClaimDetails>> {
        let response: Vec<ClaimDetails> = self.app.wrap().query_wasm_smart(
            self.contract.clone(),
            &QueryMsg::Claims {
                sender: sender.into(),
            },
        )?;
        Ok(response)
    }
}