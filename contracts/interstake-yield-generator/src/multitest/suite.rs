use anyhow::Result as AnyResult;
use schemars::JsonSchema;
use std::fmt;

use cosmwasm_std::{Addr, Coin, Decimal};
use cw_multi_test::{App, AppBuilder, AppResponse, Contract, ContractWrapper, Executor};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, TeamCommision};

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
}

impl SuiteBuilder {
    pub fn new() -> Self {
        Self {
            owner: "owner".to_owned(),
            staking_addr: "staking".to_owned(),
            team_commision: None,
            funds: vec![],
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
                router.staking.init_balance(storage, &addr, coin).unwrap();
            }
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
            staking: Addr::unchecked(self.staking_addr),
        }
    }
}

pub struct Suite {
    pub app: App,
    owner: Addr,
    contract: Addr,
    staking: Addr,
}

impl Suite {
    pub fn owner(&self) -> Addr {
        self.owner.clone()
    }

    pub fn staking(&self) -> Addr {
        self.staking.clone()
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
            &ExecuteMsg::Delegate { amount },
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
}
