#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, POLLS, Poll, BALLOTS};

const CONTRACT_NAME: &str = "crates.io:poll-ballot";
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
        ExecuteMsg::CreatePoll { id, question, answers} => exec_create_poll(deps, info, id, question, answers),
        ExecuteMsg::DeletePoll { poll_id } => exec_delete_poll(deps, info, poll_id),
        ExecuteMsg::Vote { poll_id, choice} => exec_vote(deps, info, poll_id, choice),
    } 
}

fn exec_create_poll(
    deps: DepsMut, 
    info: MessageInfo,
    id: String, 
    question: String, 
    answers: Vec<String>) -> Result<Response, ContractError>{

    if POLLS.has(deps.storage, id.clone()){
        return Err(ContractError::PollExisted)
    }

    let answer_nums = answers.len();

    if answer_nums <= 1 {
        return Err(ContractError::AnswerNotEnough{ answer_nums})
    } 

    let answers_votes = answers.iter().map(|_|  0 as u8).collect();
    let poll = Poll{
        creator: info.sender,
        question,
        answer_num: answer_nums as u8,
        answers: answers,
        answers_votes
    };

    POLLS.save(deps.storage, id, &poll)?;
    Ok(Response::new().add_attribute("action", "create_poll"))
}

fn exec_delete_poll(
    deps: DepsMut, 
    info: MessageInfo, 
    poll_id: String) -> Result<Response, ContractError>{

    if !POLLS.has(deps.storage, poll_id.clone()){
        return Err(ContractError::PollNotExisted)
    }

    let config = CONFIG.load(deps.storage).unwrap();
    let poll = POLLS.load(deps.storage, poll_id.clone()).unwrap();
    if info.sender != config.admin ||info.sender != poll.creator{
        return Err(ContractError::NoAuthority)
    }
    
    POLLS.remove(deps.storage, poll_id);
    Ok(Response::new().add_attribute("action", "delete_poll"))
}

fn exec_vote(deps: DepsMut, info: MessageInfo, poll_id: String, choice: String) -> Result<Response, ContractError> {
    if !POLLS.has(deps.storage, poll_id.clone()){
        return Err(ContractError::PollNotExisted)
    }

    let mut poll = POLLS.load(deps.storage, poll_id.clone()).unwrap();
    
    let choice_index = poll.answers.iter().position(|x| x == &choice);
    if choice_index.is_none(){
        return Err(ContractError::ChoiceNotFound { answers: choice.clone() })
    }

    let choice_index = choice_index.unwrap();
    let ballot_key = (info.sender, poll_id.clone());
    
    let mut ballot = BALLOTS.load(deps.storage, ballot_key.clone()).unwrap_or_default();
    let ballot_index = poll.answers.iter().position(|x| x == &ballot).unwrap_or_default(); 
    if BALLOTS.has(deps.storage, ballot_key.clone()){
        if ballot != choice {
            poll.answers_votes[ballot_index] -= 1;
            poll.answers_votes[choice_index] += 1;
            ballot = choice;
        }
    } else {
        poll.answers_votes[choice_index] += 1;
        ballot = choice;
    }

    POLLS.save(deps.storage, poll_id.clone(), &poll)?;
    BALLOTS.save(deps.storage, ballot_key.clone(), &ballot)?;
    Ok(Response::new().add_attribute("action", "vote"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetPoll { poll_id } => to_binary(&query_poll(deps, poll_id)),
        QueryMsg::GetBallot { voter, poll_id } => to_binary(&query_ballot(deps, voter, poll_id)),
        QueryMsg::GetVoteNum { poll_id, choice } => to_binary(&query_vote_num(deps, poll_id, choice))
    }
}

fn query_poll(deps: Deps, poll_id: String) -> Poll {
    POLLS.load(deps.storage, poll_id).unwrap()
}

fn query_ballot(deps: Deps, voter: String, poll_id: String) -> String {
    let voter = deps.api.addr_validate(&voter).unwrap();
    BALLOTS.load(deps.storage, (voter, poll_id)).unwrap()
}

fn query_vote_num(deps: Deps, poll_id: String, choice: String) -> u8{
    let poll = POLLS.load(deps.storage, poll_id).unwrap();
    
    let choice_index = poll.answers.iter().position(|x| x == &choice).unwrap();
    poll.answers_votes[choice_index]
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, attr, from_binary};

    use crate::{msg::{InstantiateMsg, ExecuteMsg, QueryMsg}, state::Poll};

    use super::{instantiate, execute, query};

    #[test]
    fn test_initiate(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        // Instantiate Contract
        let msg = InstantiateMsg{
            admin: Some(String::from("addr0"))
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        println!("Res: {:?}", res);
        assert_eq!(res.attributes, vec![
            attr("action", "instantiate")
        ]);
    }

    #[test]
    fn test_create_poll(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        // Instantiate Contract
        let msg = InstantiateMsg{
            admin: Some(String::from("addr0"))
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create poll
        let question = "What time is it?".to_string();
        let answers = vec!["7pm".to_string(), "12:30".to_string(), "9am".to_string()];
        let poll_id = "#1".to_string();
        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: poll_id.clone(),
            question: question.clone(),
            answers: answers.clone()
        };
        let resp_exe = execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();
        assert_eq!(resp_exe.attributes, vec![attr("action", "create_poll")]);
        
        // Validate poll data
        let get_poll_msg = QueryMsg::GetPoll { poll_id: poll_id};
        let query_resp = query(deps.as_ref(), env.clone(), get_poll_msg).unwrap();
        let poll_data: Poll = from_binary(&query_resp).unwrap();
        assert_eq!(poll_data.question, question);
        assert_eq!(poll_data.answers, answers);
        assert_eq!(poll_data.answer_num, 3);
        assert_eq!(poll_data.answers_votes, vec![0,0,0]);
        println!("Poll data: {:?}", poll_data);
    }

    #[test]
    #[should_panic(expected = "NoAuthority")]
    fn test_delete_poll(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        // Instantiate Contract
        let msg = InstantiateMsg{
            admin: Some(String::from("addr0"))
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create poll
        let question = "What time is it?".to_string();
        let answers = vec!["7pm".to_string(), "12:30".to_string(), "9am".to_string()];
        let poll_id = "#1".to_string();
        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: poll_id.clone(),
            question: question.clone(),
            answers: answers.clone()
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();
    
        // Delete poll
        let info = mock_info("addr2", &[]);
        let delete_poll_msg = ExecuteMsg::DeletePoll {
            poll_id : poll_id.clone()
        };
        execute(deps.as_mut(), env.clone(), info.clone(), delete_poll_msg).unwrap();
    }

    #[test]
    fn test_vote(){
        let mut deps =  mock_dependencies();
        let env = mock_env();

        let info = mock_info("addr1", &[]);
        let user = mock_info("user1", &[]);
        let user2 = mock_info("user2", &[]);
        
        // Instatiate Contract
        let msg = InstantiateMsg {
            admin: None
        };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create poll
        let question = "What time is it?".to_string();
        let answers = vec!["7pm".to_string(), "12:30".to_string(), "9am".to_string()];
        let poll_id = "#1".to_string();
        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: poll_id.clone(),
            question: question.clone(),
            answers: answers.clone()
        };
        execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();

        // Vote
        let vote_msg_1 = ExecuteMsg::Vote{poll_id: poll_id.clone(), choice: "7pm".to_string()};
        let vote_msg_2 = ExecuteMsg::Vote{poll_id: poll_id.clone(), choice: "12:30".to_string()};
        let vote_msg_3 = ExecuteMsg::Vote{poll_id: poll_id.clone(), choice: "9am".to_string()};

        let resp_vote = execute(deps.as_mut(), env.clone(), user.clone(), vote_msg_1.clone()).unwrap();
        assert_eq!(resp_vote.attributes, vec![attr("action", "vote")]);

        // Validate poll data 
        let get_choice_msg_1 = QueryMsg::GetVoteNum { poll_id: poll_id.clone(), choice: "7pm".to_string()};
        let get_choice_msg_2 = QueryMsg::GetVoteNum { poll_id: poll_id.clone(), choice: "12:30".to_string()};
        let get_choice_msg_3 = QueryMsg::GetVoteNum { poll_id: poll_id.clone(), choice: "9am".to_string()};
        
        let mut query_resp = query(deps.as_ref(), env.clone(), get_choice_msg_1.clone()).unwrap();
        let vote_num_1: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_2.clone()).unwrap();
        let vote_num_2: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_3.clone()).unwrap();
        let vote_num_3: u8 = from_binary(&query_resp).unwrap();

        assert_eq!(vote_num_1, 1);
        assert_eq!(vote_num_2, 0);
        assert_eq!(vote_num_3, 0);

        // user revote with the same choice 
        execute(deps.as_mut(), env.clone(), user.clone(), vote_msg_1.clone()).unwrap();
        
        query_resp = query(deps.as_ref(), env.clone(), get_choice_msg_1.clone()).unwrap();
        let vote_num_1: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_2.clone()).unwrap();
        let vote_num_2: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_3.clone()).unwrap();
        let vote_num_3: u8 = from_binary(&query_resp).unwrap();

        assert_eq!(vote_num_1, 1);
        assert_eq!(vote_num_2, 0);
        assert_eq!(vote_num_3, 0);

        // user change voting decision
        execute(deps.as_mut(), env.clone(), user.clone(), vote_msg_2.clone()).unwrap();
        
        query_resp = query(deps.as_ref(), env.clone(), get_choice_msg_1.clone()).unwrap();
        let vote_num_1: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_2.clone()).unwrap();
        let vote_num_2: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_3.clone()).unwrap();
        let vote_num_3: u8 = from_binary(&query_resp).unwrap();

        assert_eq!(vote_num_1, 0);
        assert_eq!(vote_num_2, 1);
        assert_eq!(vote_num_3, 0);

        // user2 vote 
        execute(deps.as_mut(), env.clone(), user2.clone(), vote_msg_3.clone()).unwrap();

        query_resp = query(deps.as_ref(), env.clone(), get_choice_msg_1.clone()).unwrap();
        let vote_num_1: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_2.clone()).unwrap();
        let vote_num_2: u8 = from_binary(&query_resp).unwrap();
        query_resp =  query(deps.as_ref(), env.clone(), get_choice_msg_3.clone()).unwrap();
        let vote_num_3: u8 = from_binary(&query_resp).unwrap();

        assert_eq!(vote_num_1, 0);
        assert_eq!(vote_num_2, 1);
        assert_eq!(vote_num_3, 1);


    }
}