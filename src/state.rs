use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr // allow the admin to delete polls 
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub data: String
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const POLLS: Map<String, Poll> = Map::new("polls");
