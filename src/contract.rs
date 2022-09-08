#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, POLLS, Poll, BALLOTS};

const CONTRACT_NAME: &str = "crates.io:cw-starter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or(info.sender.to_string()); 
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config{
        admin: validated_admin
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg { 
        ExecuteMsg::CreatePoll { id, question} => exe_create_pool(deps, info, id, question),
        ExecuteMsg::Vote { poll_id, choice} => exe_vote(deps, info, poll_id, choice)
    }
}


fn exe_create_pool(deps: DepsMut, info: MessageInfo, id: String, question: String) -> Result<Response, ContractError> {
    if POLLS.has(deps.storage, id.clone()) {
        return Err(ContractError::PollExisted{});
    }   

    let poll = Poll {
        creator: info.sender,
        question: question,
        yes_votes: 0,
        no_votes: 0
    };

    POLLS.save(deps.storage, id, &poll)?;
    Ok(Response::new().add_attribute("action", "create_poll"))
}


fn exe_vote(deps: DepsMut, info: MessageInfo, poll_id: String, choice: bool) -> Result<Response, ContractError>  {
    // Check if POLL existed 
    if !POLLS.has(deps.storage, poll_id.clone()) {
        return Err(ContractError::PollNotExisted{});
    }

    let mut poll = POLLS.load(deps.storage, poll_id.clone()).unwrap();
    let ballot_key = (info.sender.clone(), poll_id.clone());

    let mut ballot = BALLOTS.load(deps.storage, ballot_key.clone()).unwrap_or_default();
  
    if BALLOTS.has(deps.storage, ballot_key.clone()){
        if ballot != choice { 
            if ballot == true {poll.yes_votes -= 1; poll.no_votes +=1;} else {poll.yes_votes +=1; poll.no_votes -=1;};
            ballot = choice; 
        }
    }
    else {
        if choice == true {poll.yes_votes +=1} else {poll.no_votes +=1};
    }

    // Save the data 
    POLLS.save(deps.storage, poll_id.clone(), &poll)?;
    BALLOTS.save(deps.storage, ballot_key.clone(), &ballot)?;

    Ok(Response::new().add_attribute("action", "vote"))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { poll_id} => to_binary(&query_poll(deps, poll_id))
    }
}

fn query_poll (deps: Deps, poll_id: String) -> Poll { 
    POLLS.load(deps.storage, poll_id).unwrap()
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, attr, from_binary};

    use crate::{msg::{InstantiateMsg, ExecuteMsg, QueryMsg}, state::Poll};

    use super::{instantiate, execute, query};

    #[test]
    fn test_instantiate(){ 
        // Mock the environment
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        // Instantiate  
        let msg = InstantiateMsg{
            admin: Some(String::from("addr1")) // &str 
        };
        let res =  instantiate(deps.as_mut(), env, info, msg).unwrap();

        println!("Res: {:?}", res);
        assert_eq!(res.attributes, vec![
            attr("action", "instantiate")
        ]);
    }
    
    
    #[test]
    fn test_create_poll(){
        // Mock the environment 
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        // Instantiate the contract 
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()), // String, String::from("addr1")
        };
        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let question  = "Are you single?".to_string();
        let poll_id = "#1".to_string();
        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: poll_id.clone(),
            question: question.clone()
        };
        let resp_exe = execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();
        assert_eq!(resp_exe.attributes, vec![attr("action", "create_poll")]);
        

        // Validate poll data
        let get_poll_msg = QueryMsg::GetPoll { poll_id: poll_id};
        let query_resp = query(deps.as_ref(), env.clone(), get_poll_msg).unwrap();
        let poll_data: Poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll_data.question, question);
        assert_eq!(poll_data.yes_votes, 0);
        assert_eq!(poll_data.no_votes, 0);
        println!("Poll data: {:?}", poll_data);
    }

    #[test]
    fn test_vote(){
        // Mock the environment 
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);
        let user = mock_info("user1", &[]);
        let user2 = mock_info("user2", &[]);
        let user3 = mock_info("user3", &[]);
        
        // Instantiate the contract 
        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()), // String, String::from("addr1")
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let question  = "Are you single?".to_string();
        let poll_id = "#1".to_string();
        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: poll_id.clone(),
            question: question.clone()
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();

        // Vote 
        let no_vote_msg = ExecuteMsg::Vote { poll_id: poll_id.clone(), choice: false };
        let yes_vote_msg = ExecuteMsg::Vote { poll_id: poll_id.clone(), choice: true };
        let resp_vote = execute(deps.as_mut(), env.clone(), user.clone(), no_vote_msg.clone()).unwrap();
        assert_eq!(resp_vote.attributes, vec![attr("action", "vote")]);
        
        // Validate the poll data 
        let get_poll_msg = QueryMsg::GetPoll { poll_id: poll_id.clone() };
        let mut query_resp = query(deps.as_ref(), env.clone(), get_poll_msg.clone()).unwrap();
        let mut poll: Poll = from_binary(&query_resp).unwrap();

        assert_eq!(poll.no_votes, 1);
        assert_eq!(poll.yes_votes, 0);
        println!("Poll data: {:?}", poll);

        // user revote with the same choice 
        execute(deps.as_mut(), env.clone(), user.clone(), no_vote_msg.clone()).unwrap();
        query_resp = query(deps.as_ref(), env.clone(), get_poll_msg.clone()).unwrap();
        poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll.no_votes, 1);
        assert_eq!(poll.yes_votes, 0);

        println!("Poll data: {:?}", poll);

        // user change voting decision
        execute(deps.as_mut(), env.clone(), user.clone(), yes_vote_msg.clone()).unwrap();
        query_resp = query(deps.as_ref(), env.clone(), get_poll_msg.clone()).unwrap();
        poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll.no_votes, 0);
        assert_eq!(poll.yes_votes, 1);

        println!("Poll data: {:?}", poll);

        // user2 vote 
        execute(deps.as_mut(), env.clone(), user2.clone(), yes_vote_msg.clone()).unwrap();
        query_resp = query(deps.as_ref(), env.clone(), get_poll_msg.clone()).unwrap();
        poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll.no_votes, 0);
        assert_eq!(poll.yes_votes, 2);
        println!("Poll data: {:?}", poll);

        // User3 vote 
        execute(deps.as_mut(), env.clone(), user3.clone(),no_vote_msg.clone()).unwrap();
        query_resp = query(deps.as_ref(), env.clone(), get_poll_msg.clone()).unwrap();
        poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll.no_votes, 1);
        assert_eq!(poll.yes_votes, 2);
        println!("Poll data: {:?}", poll);
    }
}
