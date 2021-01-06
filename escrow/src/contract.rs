use cosmwasm_std::{
    to_binary, Api, BankMsg, Binary, CosmosMsg, Env, Extern, HandleResponse, InitResponse,
    MessageInfo, Querier, StdResult, Storage,
};

use crate::error::ContractError;
use crate::msg::{
    create_wasm_custom_msg, HandleMsg, InitMsg, MsgTransferNFT, MsgWrapper, OrderListResponse,
    QueryMsg,
};
use crate::state::{config, config_read, Order, State};
use cosmwasm_std::Coin;

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State { orders: Vec::new() };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    match msg {
        HandleMsg::CreateOrder {
            denom,
            nft_id,
            price,
        } => create_order(deps, info, denom, nft_id, price),
        HandleMsg::PayOrder { order_id } => pay_order(deps, _env, info, order_id),
    }
}

pub fn create_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: MessageInfo,
    denom: String,
    nft_id: String,
    price: Coin,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        //let order_id = time::precise_time_ns().to_string();
        state.orders.push(Order {
            id: "1234567890".to_string(),
            denom,
            nft_id,
            price,
            seller: info.sender,
        });
        Ok(state)
    })?;

    Ok(HandleResponse::default())
}

pub fn pay_order<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    order_id: String,
) -> Result<HandleResponse<MsgWrapper>, ContractError> {
    let mut msgs: Vec<CosmosMsg<MsgWrapper>> = Vec::new();
    config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
        for order in &mut state.orders {
            if order.id == order_id {
                msgs.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: order.seller.clone(),
                    amount: vec![order.price.clone()],
                }));

                let msg = MsgTransferNFT {
                    id: order.nft_id.clone(),
                    denom_id: order.denom.clone(),
                    sender: env.contract.address,
                    recipient: info.sender,
                };

                let cus = create_wasm_custom_msg(
                    String::from("/irismod.nft.MsgTransferNFT"),
                    to_binary(&msg).unwrap(),
                );
                msgs.push(cus);
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
    fn create_order() {
        let mut deps = mock_dependencies(&coins(2, "token"));
        let msg = InitMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = init(&mut deps, mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("voter1", &coins(2, "token"));
        let msg = HandleMsg::CreateOrder {
            denom: "cert".to_string(),
            nft_id: "id1".to_string(),
            price: Coin::new(100u128, "iris"),
        };
        let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(&deps, mock_env(), QueryMsg::GetOrderList {}).unwrap();
        let value: OrderListResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.list.len());
    }
}
