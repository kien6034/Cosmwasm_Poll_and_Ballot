use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("PollExisted")]
    PollExisted,
    
    #[error("PollNotExisted")]
    PollNotExisted,

    #[error("Answer can't be null")]
    AnswerCanNotBeNull,

    #[error("Answer nums must be greater than 1, current has {} answers", answer_nums)]
    AnswerNotEnough{ answer_nums: usize},

    #[error("Must be admin or poll owner to do this")]
    NoAuthority,

    #[error("Answer {} not found", answers)]
    ChoiceNotFound{ answers: String},

}
