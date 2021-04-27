use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, Deps, DepsMut, Env, HandleResponse, InitResponse,
    MessageInfo, StdResult,
};

use lifecycle_manage::{
    contract::{can_modify, destroy, freeze, init as lc_instantiate, unfreeze},
    error::ContractError as lc_Error,
    msg::InitMsg as lc_InitMsg,
};

use crate::error::ContractError;
use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> Result<InitResponse, ContractError> {
    let state = State { count: msg.count };
    config(deps.storage).save(&state)?;

    Ok(lc_instantiate(deps, env, info, lc_InitMsg {}).unwrap())
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
        HandleMsg::Increment {} => try_increment(deps, sender),
        HandleMsg::Reset { count } => try_reset(deps, count, sender),
        HandleMsg::Freeze {} => try_freeze(deps, sender),
        HandleMsg::Unfreeze {} => try_unfreeze(deps, sender),
        HandleMsg::Destroy {} => try_destroy(deps, sender),
    }
}

pub fn try_increment(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    if !can_modify(deps.as_ref(), addr) {
        return Err(ContractError::Unauthorized {});
    }
    config(deps.storage).update(|mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn try_reset(
    deps: DepsMut,
    count: i32,
    addr: CanonicalAddr,
) -> Result<HandleResponse, ContractError> {
    if !can_modify(deps.as_ref(), addr) {
        return Err(ContractError::Unauthorized {});
    }
    config(deps.storage).update(|mut state| -> Result<_, ContractError> {
        state.count = count;
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn try_freeze(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    let res = freeze(deps, addr);
    if res.is_err() {
        return Err(ContractError::Unauthorized {});
    }
    Ok(HandleResponse::default())
}

pub fn try_unfreeze(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    let res = unfreeze(deps, addr);
    if res.is_err() {
        return Err(ContractError::Unauthorized {});
    }
    Ok(HandleResponse::default())
}

pub fn try_destroy(deps: DepsMut, addr: CanonicalAddr) -> Result<HandleResponse, ContractError> {
    let res = destroy(deps, addr);
    if res.is_err() {
        return Err(ContractError::Unauthorized {});
    }
    Ok(HandleResponse::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = config_read(deps.storage).load()?;
    Ok(CountResponse { count: state.count })
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

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = HandleMsg::Increment {};
        let _res = handle(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InitMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let res = handle(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = HandleMsg::Reset { count: 5 };
        let _res = handle(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
