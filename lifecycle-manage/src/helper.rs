use cosmwasm_std::{
    CanonicalAddr, Deps, DepsMut, HandleResponse, InitResponse, MessageInfo, Order, StdResult,
};

use crate::error::ContractError;
use crate::msg::StateResponse;
use crate::state::{config, config_read, State};

pub fn init(deps: DepsMut, info: MessageInfo) -> Result<InitResponse, ContractError> {
    let state = State {
        frozen: false,
        owner: deps.api.canonical_address(&info.sender)?,
    };
    config(deps.storage).save(&state)?;
    Ok(InitResponse::default())
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

pub fn can_modify(deps: Deps, addr: CanonicalAddr) -> bool {
    query_state(deps).unwrap().state.can_modify(addr)
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = config_read(deps.storage).load()?;
    Ok(StateResponse { state })
}
