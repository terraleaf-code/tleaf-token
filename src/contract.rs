use cosmwasm_std::{Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw20_base::allowances::{execute_burn_from, execute_decrease_allowance, execute_increase_allowance, execute_send_from, execute_transfer_from, query_allowance};
use cw20_base::contract::{execute_burn, execute_mint, execute_send, execute_transfer, execute_update_marketing, execute_upload_logo, query_balance, query_download_logo, query_marketing_info, query_minter, query_token_info};
use cw20_base::contract::instantiate as cw20_instantiate;
use cw20_base::ContractError;
use cw20_base::enumerable::{query_all_accounts, query_all_allowances};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CONFIG, Config};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let cfg = Config {
        admins: msg.admins.into_iter().map(|a| deps.api.addr_validate(&a).unwrap()).collect(),
    };
    CONFIG.save(deps.storage, &cfg)?;

    cw20_instantiate(deps, env, info, Cw20InstantiateMsg {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        initial_balances: msg.initial_balances,
        mint: msg.mint,
        marketing: msg.marketing,
    })
}

// TODO: migrate and authorized burn and config and admin withdraw and set own version
// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
//     let version = get_contract_version(deps.storage)?;
//     if version.contract != CONTRACT_NAME {
//         return Err(ContractError::CannotMigrate {
//             previous_contract: version.contract,
//         });
//     }
//     Ok(Response::default())
// }

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
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Send {
            contract,
            amount,
            msg,
        } => execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::IncreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance {
            spender,
            amount,
            expires,
        } => execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom {
            owner,
            recipient,
            amount,
        } => execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom {
            owner,
            contract,
            amount,
            msg,
        } => execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::UpdateMarketing {
            project,
            description,
            marketing,
        } => execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
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
        cfg.admins = admins.into_iter().map(|a| deps.api.addr_validate(&a).unwrap()).collect();
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("sender", info.sender))
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
