use cosmwasm_std::{
    to_binary, Api, Binary, CosmosMsg, DepsMut, Empty, Env, HandleResponse, HumanAddr,
    InitResponse, MessageInfo, Querier, StdResult, Storage, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, ProxyCall, QueryMsg};
use crate::state::{config, State};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    let state = State {};
    config(deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::Register { collector } => try_register(deps, env, collector),
    }
}

pub fn try_register(
    _deps: DepsMut,
    _env: Env,
    collector: HumanAddr,
) -> Result<HandleResponse, ContractError> {
    let msg = to_binary(&ProxyCall { call: Empty {} })?;
    let msgs = vec![CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: collector,
        msg: msg,
        send: vec![],
    })];
    Ok(HandleResponse {
        messages: msgs,
        data: None,
        attributes: vec![],
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: DepsMut,
    _env: Env,
    _msg: QueryMsg,
) -> StdResult<Binary> {
    match _msg {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{to_binary, Empty};

    #[test]
    fn try_register() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("anyone", &coins(2, "token"));
        let msg = HandleMsg::Register {
            collector: HumanAddr::from("collector"),
        };
        let res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());

        let msg = to_binary(&ProxyCall { call: Empty {} }).unwrap();
        println!("{}", msg)
    }
}
