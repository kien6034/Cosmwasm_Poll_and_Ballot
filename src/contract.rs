#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, POLLS, Poll};

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


fn exe_vote(deps: DepsMut, _info: MessageInfo, question_id: String, choice: bool) -> Result<Response, ContractError>  {
    // Check if POLL existed 
    if !POLLS.has(deps.storage, question_id.clone()) {
        return Err(ContractError::PollNotExisted{});
    }

    // Determine choice
    POLLS.update(
        deps.storage,
        question_id, 
        |poll: Option<Poll>| -> StdResult<_> {
            let mut poll_data = poll.unwrap();
            if choice == false {poll_data.no_votes += 1} else {poll_data.yes_votes+=1}
            Ok(poll_data)
        }
    )?;

    Ok(Response::new().add_attribute("method", "exe_vote"))
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
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        //println!("Addr: {}", info.sender);
        let msg = InstantiateMsg{
            admin: Some(String::from("addr1")) // &str 
        };

        let res =  instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(res.attributes, vec![
            attr("action", "instantiate")
        ]);
    }
    
    
    #[test]
    fn test_create_poll(){
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()), // String, String::from("addr1")
        };

        // Before you execute a contract you need to instantiate it
        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: "#1".to_string(),
            question: "Do you love Spark IBC?".to_string(),
        };

        let _resp_exe = execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();

        let get_poll_msg = QueryMsg::GetPoll { poll_id: "#1".to_string() };
        let query_resp = query(deps.as_ref(), env.clone(), get_poll_msg).unwrap();

        let poll_data: Poll = from_binary(&query_resp).unwrap();
        println!("Poll data: {:?}", poll_data);
    }

    #[test]
    fn test_vote(){
        let mut deps =  mock_dependencies();
        let env = mock_env();
        let info = mock_info("addr1", &[]);

        let user = mock_info("user1", &[]);

        let msg = InstantiateMsg {
            admin: Some("addr1".to_string()), // String, String::from("addr1")
        };

        // Before you execute a contract you need to instantiate it
        let _resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let create_poll_msg = ExecuteMsg::CreatePoll {
            id: "#1".to_string(),
            question: "Do you love Spark IBC?".to_string(),
        };

        let _resp_exe = execute(deps.as_mut(), env.clone(), info.clone(), create_poll_msg).unwrap();

       

        //vote 
        let vote_msg = ExecuteMsg::Vote { poll_id: "#1".to_string(), choice: false };
        let _resp_vote = execute(deps.as_mut(), env.clone(), user.clone(), vote_msg.clone()).unwrap();
        execute(deps.as_mut(), env.clone(), user.clone(), vote_msg.clone()).unwrap();
        execute(deps.as_mut(), env.clone(), user.clone(), vote_msg).unwrap();

        let get_poll_msg = QueryMsg::GetPoll { poll_id: "#1".to_string() };
        let query_resp = query(deps.as_ref(), env.clone(), get_poll_msg).unwrap();

        let poll_data: Poll = from_binary(&query_resp).unwrap();
        println!("Poll data: {:?}", poll_data);

    }
}
