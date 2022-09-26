use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("PollExisted")]
    PollExisted,
    
    #[error("PollNotExisted")]
    PollNotExisted,

    #[error("OptionNotExisted")]
    OptionNotExisted
}
