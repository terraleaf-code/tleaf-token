use cosmwasm_std::{Addr, BankMsg, Binary, coin, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsg, to_binary, Uint128};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::{get_contract_version, set_contract_version};
use cw20::{Logo, LogoInfo, MarketingInfoResponse};
use terra_cosmwasm::TerraQuerier;

use cw20_base::allowances::{execute_decrease_allowance, execute_increase_allowance, execute_send_from, execute_transfer_from, query_allowance};
use cw20_base::contract::{create_accounts, execute_mint, execute_send, execute_transfer, execute_update_marketing, execute_upload_logo, query_balance, query_download_logo, query_marketing_info, query_minter, query_token_info};
use cw20_base::contract::execute_burn as cw20_execute_burn;
use cw20_base::allowances::execute_burn_from as cw20_execute_burn_from;
use cw20_base::enumerable::{query_all_accounts, query_all_allowances};
use cw20_base::state::{LOGO, MARKETING_INFO, MinterData, TOKEN_INFO, TokenInfo};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{CONFIG, Config};
use crate::error::ContractError;

static DECIMAL_FRACTION: Uint128 = Uint128::new(1_000_000_000_000_000_000u128);

// version info for migration info
const CONTRACT_NAME: &str = "terraleaf.io:tleaf-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        admins: msg.admins.into_iter().map(|a| deps.api.addr_validate(&a).unwrap()).collect(),
    };
    CONFIG.save(deps.storage, &cfg)?;

    let msg = Cw20InstantiateMsg{
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: msg.initial_balances,
        mint: msg.mint,
        marketing: msg.marketing
    };

    // check valid token info
    msg.validate()?;
    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances)?;

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(StdError::generic_err("Initial supply greater than cap").into());
        }
    }

    let mint = match msg.mint {
        Some(m) => Some(MinterData {
            minter: deps.api.addr_validate(&m.minter)?,
            cap: m.cap,
        }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    if let Some(marketing) = msg.marketing {
        let logo = if let Some(logo) = marketing.logo {
            LOGO.save(deps.storage, &logo)?;

            match logo {
                Logo::Url(url) => Some(LogoInfo::Url(url)),
                Logo::Embedded(_) => Some(LogoInfo::Embedded),
            }
        } else {
            None
        };

        let data = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing
                .marketing
                .map(|addr| deps.api.addr_validate(&addr))
                .transpose()?,
            logo,
        };
        MARKETING_INFO.save(deps.storage, &data)?;
    }

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { admins } =>
            execute_update_config(deps, info, admins),
        ExecuteMsg::Transfer { recipient, amount } => {
            Ok(execute_transfer(deps, env, info, recipient, amount)?)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => Ok(execute_send(deps, env, info, contract, amount, msg)?),
        ExecuteMsg::Mint { recipient, amount } => Ok(execute_mint(deps, env, info, recipient, amount)?),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_increase_allowance(deps, env, info, spender, amount, expires)?),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => Ok(execute_decrease_allowance(deps, env, info, spender, amount, expires)?),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => Ok(execute_transfer_from(deps, env, info, owner, recipient, amount)?),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => Ok(execute_send_from(deps, env, info, owner, contract, amount, msg)?),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => Ok(execute_update_marketing(deps, env, info, project, description, marketing)?),
        ExecuteMsg::UploadLogo(logo) => Ok(execute_upload_logo(deps, env, info, logo)?),
        ExecuteMsg::WithdrawLockedFunds {
            denom,
            amount,
            recipient
        } => execute_withdraw_locked_funds(deps, info, denom, amount, recipient)
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_admins: Option<Vec<String>>,
) -> Result<Response, ContractError> {
    // only authorized admins can perform this action
    let mut cfg = CONFIG.load(deps.storage)?;
    if !is_admin(&cfg, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admins) = new_admins {
        if admins.len() == 0 {
            return Err(ContractError::Std(StdError::generic_err(
                "At least one admin required",
            )));
        }

        cfg.admins = admins.into_iter().map(|a| deps.api.addr_validate(&a).unwrap()).collect();
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("sender", info.sender))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // only authorized admins can perform this action
    let cfg = CONFIG.load(deps.storage)?;
    if !is_admin(&cfg, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    Ok(cw20_execute_burn(deps, env, info, amount)?)
}

pub fn execute_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // only authorized admins can perform this action
    let cfg = CONFIG.load(deps.storage)?;
    if !is_admin(&cfg, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    Ok(cw20_execute_burn_from(deps, env, info, owner, amount)?)
}

pub fn execute_withdraw_locked_funds(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    amount: Uint128,
    recipient: String,
) -> Result<Response, ContractError> {
    // authorized owner
    let cfg = CONFIG.load(deps.storage)?;
    if !is_admin(&cfg, &info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attribute("method", "withdraw_locked_funds")
        .add_attribute("sender", info.sender.clone())
        .add_attribute("denom", denom.clone())
        .add_attribute("amount", amount.to_string())
        .add_attribute("recipient", recipient.clone())
        .add_submessage(SubMsg::new(BankMsg::Send {
            to_address: recipient,
            amount: vec![deduct_tax(deps, coin(amount.u128(), denom))?],
        })))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances {
            owner,
            start_after,
            limit,
        } => to_binary(&query_all_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<Config> {
    Ok(CONFIG.load(deps.storage)?)
}

fn is_admin(cfg: &Config, addr: &Addr) -> bool {
    return cfg.admins.iter().any(|a| a == addr);
}

fn compute_tax(deps: DepsMut, coin: &Coin) -> Result<Uint128, ContractError> {
    let amount = coin.amount;
    let denom = coin.denom.clone();

    if denom == "uluna" {
        Ok(Uint128::zero())
    } else {
        let terra_querier = TerraQuerier::new(&deps.querier);
        let tax_rate: Decimal = (terra_querier.query_tax_rate()?).rate;
        let tax_cap: Uint128 = (terra_querier.query_tax_cap(denom.to_string())?).cap;
        Ok(std::cmp::min(
            amount.checked_sub(amount.multiply_ratio(
                DECIMAL_FRACTION,
                DECIMAL_FRACTION * tax_rate + DECIMAL_FRACTION,
            ))?,
            tax_cap,
        ))
    }
}

fn deduct_tax(deps: DepsMut, coin: Coin) -> Result<Coin, ContractError> {
    Ok(Coin {
        denom: coin.denom.clone(),
        amount: coin.amount.checked_sub(compute_tax(deps, &coin)?)?,
    })
}
