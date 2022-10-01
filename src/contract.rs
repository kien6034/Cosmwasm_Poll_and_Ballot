#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use crate::state::{Ballot, Config, Poll, BALLOTS, CONFIG, POLLS};

use crate::error::ContractError;
use crate::msg::{
    AllPollsResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
};

const CONTRACT_NAME: &str = "crates.io:cw-starter:poll-ballot-multi-options";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // unimplemented!()
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = _msg.admin.unwrap_or(_info.sender.to_string()); // if None, use info.sender
    let validated_admin = _deps.api.addr_validate(&admin)?; // validate the address
    let config = Config {
        admin: validated_admin.clone(),
    };
    CONFIG.save(_deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", validated_admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // unimplemented!()
    match _msg {
        ExecuteMsg::CreatePoll { uuid, question, options } => execute_create_poll(_deps, _env, _info, uuid, question, options),
        ExecuteMsg::Vote { uuid, option } => execute_vote(_deps, _env, _info, uuid, option),
    }
}

fn execute_create_poll(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    uuid: String,
    question: String,
    options: Vec<String>,
) -> Result<Response, ContractError> {
    // Polls can be defined with many different options
    // - The maximum options defined will be 10 due to gas limitations
    if options.len() > 10 {
        return Err(ContractError::TooManyOptions {});
    }
    // Create List [Option - count vote]
    let mut opts: Vec<(String, u64)> = vec![];
    // Add Item to list with option and count vote default 0
    for option in options {
        opts.push((option, 0));
    }
    // Poll set default 
    let poll = Poll {
        creator: _info.sender,
        question,
        options: opts,
    };
    // save store , check in state.rs:44 
    POLLS.save(_deps.storage, uuid, &poll)?;

    Ok(Response::new())
}

fn execute_vote(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    uuid: String,
    option: String,
) -> Result<Response, ContractError> {

    let poll = POLLS.may_load(_deps.storage, uuid.clone())?;
    
    match poll {
        // if pool exists
        Some(mut poll) => {
            BALLOTS.update(
                _deps.storage,
                (_info.sender, uuid.clone()), // get Ballot of Poll with user vote 
                |ballot| -> StdResult<Ballot> {
                    match ballot {
                        // If user Vote ready - update vote
                        Some(ballot) => {
                            // Find position of opiton in list 
                            let position_of_old_vote = poll
                                .options
                                .iter()
                                .position(|option| option.0 == ballot.option)
                                .unwrap();
                            // User change option , -1 old option in Poll
                            poll.options[position_of_old_vote].1 -= 1;
                            // Update the ballot
                            Ok(Ballot {
                                option: option.clone(),
                            })
                        }
                        // If user not vote - add vote
                        None => {
                            // Simply add the ballot
                            Ok(Ballot {
                                option: option.clone(),
                            })
                        }
                    }
                },
            )?;

            // Find the position of the new vote option and increment it by 1
            let position = poll.options.iter().position(|poll_option| poll_option.0 == option);
            if position.is_none() {
                return Err(ContractError::Unauthorized {});
            }
            let position = position.unwrap();
            // Update Count Option + 1
            poll.options[position].1 += 1;
            // Update Poll in store
            POLLS.save(_deps.storage, uuid, &poll)?;
            Ok(Response::new())
        }
        // if pool not exists
        None => Err(ContractError::Unauthorized {}), // The poll does not exist so we just error
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    // unimplemented!()
    match _msg {
        QueryMsg::AllPolls {} => query_all_polls(_deps, _env),
        QueryMsg::GetPoll { uuid } => query_poll(_deps, _env, uuid),
        QueryMsg::GetVote { address, uuid } => query_vote(_deps, _env, address, uuid),
    }
}

fn query_all_polls(_deps: Deps, _env: Env) -> StdResult<Binary> {
    let polls = POLLS
        .range(_deps.storage, None, None, Order::Ascending)
        .map(|p| Ok(p?.1))
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&AllPollsResponse { polls })
}

fn query_poll(_deps: Deps, _env: Env, uuid: String) -> StdResult<Binary> {
    let poll = POLLS.may_load(_deps.storage, uuid)?;
    to_binary(&PollResponse { poll })
}

fn query_vote(_deps: Deps, _env: Env, address: String, uuid: String) -> StdResult<Binary> {
    let validated_address = _deps.api.addr_validate(&address).unwrap();
    let ballot = BALLOTS.may_load(_deps.storage, (validated_address, uuid))?;

    to_binary(&VoteResponse { ballot })
}

#[cfg(test)]
mod tests {
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{
        AllPollsResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{attr, from_binary};

    pub const ADDR1: &str = "addr1";
    #[test]
    fn test_instantiate_addr1() {
        // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
        let mut deps = mock_dependencies();
        // Mock the contract environment, contains the block info, contract address, etc.
        let env = mock_env();
        // Mock the message info, ADDR1 will be the sender, the empty vec means we sent no funds.
        let info = mock_info(ADDR1, &[]);

        // Create a message where we (the sender) will be an admin
        let msg = InstantiateMsg { admin: None };
        // Call instantiate, unwrap to assert success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR1)]
        )
    }
    pub const ADDR2: &str = "addr2";
    #[test]
    fn test_instantiate_addr2() {
        // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
        let mut deps = mock_dependencies();
        // Mock the contract environment, contains the block info, contract address, etc.
        let env = mock_env();
        // Mock the message info, ADDR2 will be the sender, the empty vec means we sent no funds.
        let info = mock_info(ADDR2, &[]);

        // Create a message where we (the sender) will be an admin
        let msg = InstantiateMsg { admin: None };
        // Call instantiate, unwrap to assert success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR2)]
        )
    }
    #[test]
    fn test_instantiate_addr2_with_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        // Send as ADDR1 to show admin is different
        let info = mock_info(ADDR1, &[]);

        // Create a message where ADDR2 will be an admin
        // Have to use .to_string() method
        let msg = InstantiateMsg {
            admin: Some(ADDR2.to_string()),
        };
        // Unwrap to assert success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        // Assert admin is ADDR2 instead
        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR2),]
        );
    }
    #[test]
    fn test_execute_create_poll_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // New execute msg
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };

        // Unwrap to assert success
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    }
    #[test]
    fn test_execute_create_poll_invalid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            uuid: "some_id".to_string(),
            question: "What's your favourite number?".to_string(),
            options: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
                "11".to_string(),
            ],
        };

        // Unwrap error to assert failure
        let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    }
    #[test]
    fn test_execute_vote_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the vote, first time voting
        let msg = ExecuteMsg::Vote {
            uuid: "uuid".to_string(),
            option: "Rust".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Change the vote
        let msg = ExecuteMsg::Vote {
            uuid: "uuid".to_string(),
            option: "Go".to_string(),
        };
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();
    }
    #[test]
    fn test_execute_vote_invalid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the vote, some_id poll is not created yet.
        let msg = ExecuteMsg::Vote {
            uuid: "uuid".to_string(),
            option: "Rust".to_string(),
        };
        // Unwrap to assert error
        let _err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        // Create the poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Vote on a now existing poll but the option "Java" does not exist
        let msg = ExecuteMsg::Vote {
            uuid: "uuid".to_string(),
            option: "Java".to_string(),
        };
        let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    }
    #[test]
    fn test_query_all_polls() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a second poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid_uuid".to_string(),
            question: "What's your favourite number?".to_string(),
            options: vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query
        let msg = QueryMsg::AllPolls {};
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: AllPollsResponse = from_binary(&bin).unwrap();
        assert_eq!(res.polls.len(), 2);
    }
    #[test]
    fn test_query_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query for the poll that exists
        let msg = QueryMsg::GetPoll {
            uuid: "uuid".to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: PollResponse = from_binary(&bin).unwrap();
        // Expect a poll
        assert!(res.poll.is_some());

        // Query for the poll that does not exists
        let msg = QueryMsg::GetPoll {
            uuid: "uuid_id_not_exist".to_string(),
        };
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: PollResponse = from_binary(&bin).unwrap();
        // Expect none
        assert!(res.poll.is_none());
    }
    #[test]
    fn test_query_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            uuid: "uuid".to_string(),
            question: "What's your favourite programming language?".to_string(),
            options: vec![
                "Rust".to_string(),
                "Go".to_string(),
                "JavaScript".to_string(),
                "Haskell".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a vote
        let msg = ExecuteMsg::Vote {
            uuid: "uuid".to_string(),
            option: "Rust".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query for a vote that exists
        let msg = QueryMsg::GetVote {
            uuid: "uuid".to_string(),
            address: ADDR1.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: VoteResponse = from_binary(&bin).unwrap();
        // Expect the vote to exist
        assert!(res.ballot.is_some());

        // Query for a vote that does not exists
        let msg = QueryMsg::GetVote {
            uuid: "uuid_2".to_string(),
            address: ADDR2.to_string(),
        };
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: VoteResponse = from_binary(&bin).unwrap();
        // Expect the vote to not exist
        assert!(res.ballot.is_none());
    }
}
