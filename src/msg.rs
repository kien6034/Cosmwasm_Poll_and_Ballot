use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::{Ballot, Poll};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: Option<String>, // If admin is None, default to the sender's address
}

// Any user can create a poll
// - Polls are identified by a unique string, (this could be a UUID or a slug)
// Any user can vote on a poll
// - Ballots are stored per poll per address, meaning a user's vote can be updated any time
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreatePoll {
        uuid: String,
        question: String,
        options: Vec<String>
    },
    Vote {
        uuid: String,
        option: String
    }
}

// Any user can query and retrieve all the polls
// Any user can query and retrieve one poll by its unique identifier
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPoll{
        uuid: String,
    },
    GetBallot{
        uuid: String,
        voter: String
    },
    GetVote{
        uuid: String,
        address: String
    }
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub struct PollResponse {
//     val: String,
// }

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AllPollsResponse {
    pub polls: Vec<Poll>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct PollResponse {
    pub options: Option<Poll>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct VoteResponse {
    pub ballots: Option<Ballot>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
