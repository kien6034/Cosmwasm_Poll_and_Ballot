use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr, // allow the admin to delete polls
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub creator: Addr,
    pub question: String,
    pub options: Vec<(String, u64)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ballot {
    pub option: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

// A map with a String key and Poll value
// The key must be unique, this could be a UUID or a generated slug
pub const POLLS: Map<&str, Poll> = Map::new("polls");

// A map with a tuple key (Addr, String) and a Ballot value
// The tuple is made up of the voter's address and the polls ID
// Example:
// A key of ("wasm1xxx", "1") will point to the vote of address
// wasm1xxx for poll 1
pub const BALLOTS: Map<(Addr, &str), Ballot> = Map::new("ballots");
