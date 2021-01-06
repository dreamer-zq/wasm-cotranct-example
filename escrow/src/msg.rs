use cosmwasm_std::CosmosMsg;
use cosmwasm_std::{Binary, Coin, HumanAddr};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::Order;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreateOrder {
        denom: String,
        nft_id: String,
        price: Coin,
        name: String,
        uri: String,
        data: String,
    },
    PayOrder {
        order_no: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetOrderList returns the current count as a json-encoded number
    GetOrderList {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OrderListResponse {
    pub list: Vec<Order>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgMintNFT {
    pub id: String,
    pub denom_id: String,
    pub name: String,
    pub uri: String,
    pub data: String,
    pub sender: HumanAddr,
    pub recipient: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgTransferNFT {
    pub id: String,
    pub denom_id: String,
    pub sender: HumanAddr,
    pub recipient: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgWrapper {
    pub router: String,
    pub data: String,
}

// this is a helper to be able to return these as CosmosMsg easier
impl Into<CosmosMsg<MsgWrapper>> for MsgWrapper {
    fn into(self) -> CosmosMsg<MsgWrapper> {
        CosmosMsg::Custom(self)
    }
}

// create_swap_msg returns wrapped swap msg
pub fn create_wasm_custom_msg(typ: String, value: Binary) -> CosmosMsg<MsgWrapper> {
    MsgWrapper {
        router: typ,
        data: value.to_string(),
    }
    .into()
}
