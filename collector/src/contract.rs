use cosmwasm_std::{
    to_binary, Binary, DepsMut, Env, HandleResponse, InitResponse, MessageInfo, StdResult,
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, QueryCallersResponse, QueryMsg};
use crate::state::{config, config_read, State};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    let state = State {
        callers: Vec::new(),
    };
    config(deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        HandleMsg::Call {} => try_call(deps, info),
    }
}

pub fn try_call(deps: DepsMut, info: MessageInfo) -> Result<HandleResponse, ContractError> {
    config(deps.storage).update(|mut state| -> Result<_, ContractError> {
        if !state.callers.contains(&info.sender) {
            state.callers.push(info.sender);
        }
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn query(deps: DepsMut, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCallers {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: DepsMut) -> StdResult<QueryCallersResponse> {
    let state = config_read(deps.storage).load()?;
    Ok(QueryCallersResponse {
        callers: state.callers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn call() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = HandleMsg::Call {};
        let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_mut(), mock_env(), QueryMsg::GetCallers {}).unwrap();
        let value: QueryCallersResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.callers.len());
    }
}
