use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr // allow the admin to delete polls
}

// 1. User A creates a poll it has the following details:
// - "What is your favourite programming language?"
// ---- Rust
// ---- Go
// ---- JavaScript
// ---- Haskell
// 2. User A decides to vote on their own poll, they vote for Rust
// - Rust now has 1 vote, the rest have 0 still
// 3. User B also decides to vote, they vote for Go
// - Go now has 1 vote, Rust also has 1 vote, the rest have 0 votes
// 4. Some time passes and User A decides to change their vote to Haskell
// - Rust now has 0 votes (as User A's vote has been changed), Haskell has 1 vote (User A's new vote), Go still has 1 vote and the rest are on 0
// 5. User C decides to query the poll and check the results they see the following:
// - "What is your favourite programming language?"
// ---- Rust - 0
// ---- Go - 1
// ---- JavaScript - 0
// ---- Haskell - 1

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub creator: Addr, // User create Poll
    pub question: String,
    pub options: Vec<(String, u64)>, // option name - vote count
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ballot {
    pub option: String
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const POLLS: Map<String, Poll> = Map::new("polls");
pub const BALLOTS: Map<(Addr, String), Ballot> = Map::new("ballots");
