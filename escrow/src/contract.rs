use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, CosmosMsg, Env, Extern, HandleResponse, InitResponse,
    MessageInfo, Querier, StdResult, Storage,
};

use crate::error::ContractError;
use crate::msg::{
    create_wasm_custom_msg, HandleMsg, InitMsg, MsgMintNFT, MsgTransferNFT, MsgWrapper,
    OrderListResponse, QueryMsg,
};
use crate::state::{config, config_read, Order, OrderState, State};
use cosmwasm_std::{has_coins, Coin};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        orders: Vec::new(),
        sequence: 1,
    };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    match msg {
        HandleMsg::Create {
            denom,
            nft_id,
            name,
            uri,
            data,
            price,
        } => place_order(deps, env, info, denom, nft_id, name, uri, data, price),
        HandleMsg::Delegated {
            denom,
            nft_id,
            price,
        } => delegated_order(deps, env, info, denom, nft_id, price),
        HandleMsg::Pay { order_no } => pay_order(deps, env, info, order_no),
        HandleMsg::Cancel { order_no } => cancel_order(deps, env, info, order_no),
    }
}

pub fn place_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    denom: String,
    nft_id: String,
    name: String,
    uri: String,
    data: String,
    price: Coin,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    let mut msgs: Vec<CosmosMsg<MsgWrapper>> = Vec::new();

    config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        state.orders.push(Order {
            no: state.sequence.to_string(),
            denom: denom.clone(),
            nft_id: nft_id.clone(),
            price,
            seller: info.sender,
            buyer: Default::default(),
            state: OrderState::PENDING,
        });

        let msg = MsgMintNFT {
            id: nft_id.clone(),
            denom_id: denom.clone(),
            name,
            uri,
            data,
            sender: env.contract.address.clone(),
            recipient: env.contract.address,
        };

        let data = create_wasm_custom_msg(
            String::from("/irismod.nft.MsgMintNFT"),
            to_binary(&msg).unwrap(),
        );
        msgs.push(data);

        state.sequence = state.sequence + 1;
        Ok(state)
    })?;

    let r = HandleResponse {
        messages: msgs,
        data: None,
        attributes: vec![],
    };
    Ok(r)
}

pub fn delegated_order<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    _denom: String,
    _nft_id: String,
    _price: Coin,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    //TODO
    Ok(HandleResponse::default())
}

pub fn cancel_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    order_no: String,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    let mut msgs: Vec<CosmosMsg<MsgWrapper>> = Vec::new();
    config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        for order in &mut state.orders {
            if order.no == order_no {
                if order.state != OrderState::PENDING {
                    return Err(ContractError::InvalidOrderState {
                        order_id: order.no.clone(),
                    });
                }

                if order.seller != info.sender {
                    return Err(ContractError::InvalidOrderState {
                        order_id: order.no.clone(),
                    });
                }

                order.state = OrderState::REVOKE;

                let msg = MsgTransferNFT {
                    id: order.nft_id.clone(),
                    denom_id: order.denom.clone(),
                    name: "[do-not-modify]".to_string(),
                    data: "[do-not-modify]".to_string(),
                    uri: "[do-not-modify]".to_string(),
                    sender: env.contract.address.clone(),
                    recipient: info.sender.clone(),
                };

                let data = create_wasm_custom_msg(
                    String::from("/irismod.nft.MsgTransferNFT"),
                    to_binary(&msg).unwrap(),
                );
                msgs.push(data);
            }
        }
        Ok(state)
    })?;
    let r = HandleResponse {
        messages: msgs,
        data: None,
        attributes: vec![],
    };
    Ok(r)
}

pub fn pay_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    order_no: String,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    if info.sent_funds.len() == 0 {
        return Err(ContractError::InvalidRequest { order_id: order_no });
    }

    let mut msgs: Vec<CosmosMsg<MsgWrapper>> = Vec::new();
    config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        for order in &mut state.orders {
            if order.no == order_no {
                if order.state != OrderState::PENDING {
                    return Err(ContractError::InvalidOrderState {
                        order_id: order.no.clone(),
                    });
                }

                if !has_coins(&info.sent_funds, &order.price) {
                    return Err(ContractError::InvalidRequest { order_id: order_no });
                }

                order.state = OrderState::PAID;
                order.buyer = info.sender.clone();

                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: order.seller.clone(),
                    amount: vec![order.price.clone()],
                }));

                let msg = MsgTransferNFT {
                    id: order.nft_id.clone(),
                    denom_id: order.denom.clone(),
                    name: "[do-not-modify]".to_string(),
                    data: "[do-not-modify]".to_string(),
                    uri: "[do-not-modify]".to_string(),
                    sender: env.contract.address,
                    recipient: info.sender,
                };

                let data = create_wasm_custom_msg(
                    String::from("/irismod.nft.MsgTransferNFT"),
                    to_binary(&msg).unwrap(),
                );
                msgs.push(data);
                break;
            }
        }
        Ok(state)
    })?;

    let r = HandleResponse {
        messages: msgs,
        data: None,
        attributes: vec![],
    };
    Ok(r)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOrderList {} => to_binary(&query_order_list(deps)?),
    }
}

fn query_order_list<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<OrderListResponse> {
    let state = config_read(&deps.storage).load()?;
    //let order_list = state.orders.values().cloned().collect::<Vec<Order>>();
    Ok(OrderListResponse { list: state.orders })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn place_order() {
        let mut deps = mock_dependencies(&coins(2, "token"));
        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "iris"));
        let _res = init(&mut deps, mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("voter1", &coins(2, "iris"));
        let msg = HandleMsg::Create {
            denom: "cert".to_string(),
            nft_id: "id1".to_string(),
            name: "test".to_string(),
            uri: "test".to_string(),
            data: "test".to_string(),
            price: Coin::new(100u128, "iris"),
        };
        let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, mock_env(), QueryMsg::GetOrderList {}).unwrap();
        let value: OrderListResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.list.len());

        let msg = HandleMsg::Pay {
            order_no: "1".to_string(),
        };

        // beneficiary can release it
        let info = mock_info("voter2", &coins(100, "iris"));
        let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, mock_env(), QueryMsg::GetOrderList {}).unwrap();
        let value: OrderListResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.list.len());

        let order = value.list.get(0).unwrap();
        assert_eq!(OrderState::PAID, order.state);
    }
}
