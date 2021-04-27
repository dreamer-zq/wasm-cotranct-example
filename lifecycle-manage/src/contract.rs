use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, Deps, DepsMut, Env, HandleResponse, InitResponse,
    MessageInfo, Order, StdResult,
};

use crate::error::ContractError;
use crate::msg::{HandleMsg, InitMsg, QueryMsg, StateResponse};
use crate::state::{config, config_read, State};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InitMsg,
) -> Result<InitResponse, ContractError> {
    let state = State {
        frozen: false,
        owner: deps.api.canonical_address(&info.sender)?,
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
    let sender = deps.api.canonical_address(&info.sender)?;
    match msg {
        HandleMsg::Freeze {} => freeze(deps, sender),
        HandleMsg::Unfreeze {} => unfreeze(deps, sender),
        HandleMsg::Destroy {} => destroy(deps, sender),
    }
}

pub fn freeze(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    config(deps.storage).update(|mut state| -> Result<_, ContractError> {
        if !state.can_modify(addr) {
            return Err(ContractError::Unauthorized {});
        }
        state.frozen = true;
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn unfreeze(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    config(deps.storage).update(|mut state| -> Result<_, ContractError> {
        if !state.is_owner(addr) {
            return Err(ContractError::Unauthorized {});
        }

        if !state.frozen {
            return Err(ContractError::InvalidOperate {});
        }
        state.frozen = false;
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn destroy(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    let state = config_read(deps.storage).load()?;
    if !state.is_owner(addr) {
        return Err(ContractError::Unauthorized {});
    }

    // delete all state
    let keys: Vec<_> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(k, _)| k)
        .collect();
    for k in keys {
        deps.storage.remove(&k);
    }
    Ok(HandleResponse::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = config_read(deps.storage).load()?;
    Ok(StateResponse { state })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let value: StateResponse = from_binary(&res).unwrap();
        assert_eq!(false, value.state.frozen);
    }

    #[test]
    fn freeze() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("creator", &coins(2, "token"));
        let msg = HandleMsg::Freeze {};
        let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let value: StateResponse = from_binary(&res).unwrap();
        assert_eq!(true, value.state.frozen);
    }

    #[test]
    fn unfreeze() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("creator", &coins(2, "token"));
        let msg = HandleMsg::Freeze {};
        let _res = handle(deps.as_mut(), mock_env(), unauth_info, msg);

        // only the original creator can unfreeze the contract
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = HandleMsg::Unfreeze {};
        let _res = handle(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be Normal
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let value: StateResponse = from_binary(&res).unwrap();
        assert_eq!(false, value.state.frozen);
    }

    #[test]
    fn destroy() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can destroy it
        let info = mock_info("creator", &coins(2, "token"));
        let msg = HandleMsg::Destroy {};
        let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetState {});
        //let res: StdResult<HandleResponse> = from_binary(&res);
        match res.unwrap_err() {
            StdError::NotFound { kind, .. } => {
                assert_eq!(kind, "lifecycle_manage::state::State")
            }
            _ => panic!("expected migrate error message"),
        }
    }
}
