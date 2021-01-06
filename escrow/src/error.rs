use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("({order_id}) Order not found")]
    OrderNotExist { order_id: String },

    #[error("({order_id}) Order state invalid")]
    InvalidOrderState { order_id: String },

    #[error("You should be paid for ({order_id}) order")]
    InvalidRequest { order_id: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
