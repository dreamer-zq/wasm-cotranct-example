#[cfg(not(feature = "library"))]
use std::collections::HashMap;

use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Vote, VoteResponse};
use crate::state::{State, VoteInfo, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:election";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        start: msg.start,
        end: msg.end,
        candidates: msg.candidates,
        votes: Vec::new(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

// And declare a custom Error variant for the ones where you will want to make use of it

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Vote { candidate } => try_vote(deps, env, info, candidate),
    }
}

pub fn try_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    candidate: String,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if env.block.height < state.start || env.block.height > state.end {
            return Err(ContractError::NotAllowance {
                begin: state.start,
                end: state.end,
            });
        }
        state.votes.push(VoteInfo {
            voter: info.sender.to_string(),
            candidate: candidate,
        });
        Ok(state)
    })?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVoteInfo {} => to_binary(&query_vote_info(deps)?),
    }
}

fn query_vote_info(deps: Deps) -> StdResult<VoteResponse> {
    let state = STATE.load(deps.storage)?;
    let mut vote_info = HashMap::new();
    for vote in state.votes {
        let count = vote_info.entry(vote.candidate).or_insert(0);
        *count += 1;
    }

    let mut votes = Vec::new();
    for (candidate, count) in vote_info {
        votes.push(Vote {
            candidate: candidate,
            count: count,
        });
    }
    Ok(VoteResponse {
        votes: votes,
        start: state.start,
        end: state.end,
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

        let msg = InstantiateMsg {
            start: 10,
            end: 100,
            candidates: Vec::new(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVoteInfo {}).unwrap();
        let value: VoteResponse = from_binary(&res).unwrap();
        assert_eq!(10, value.start);
        assert_eq!(100, value.end);
    }

    #[test]
    fn vote() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let mut candidates: Vec<String> = Vec::new();
        candidates.push("candidates1".into());
        candidates.push("candidates2".into());
        let msg = InstantiateMsg {
            start: 10_000,
            end: 20_000,
            candidates: Vec::new(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("voter1", &coins(2, "token"));
        let msg = ExecuteMsg::Vote {
            candidate: "candidates1".into(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetVoteInfo {}).unwrap();
        let value: VoteResponse = from_binary(&res).unwrap();
        assert_eq!(10_000, value.start);
        assert_eq!(20_000, value.end);
        assert_eq!("candidates1", value.votes[0].candidate);
        assert_eq!(1, value.votes[0].count);
    }
}
