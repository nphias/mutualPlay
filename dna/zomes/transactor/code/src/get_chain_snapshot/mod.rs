use hdk::holochain_core_types::chain_header::ChainHeader;
use hdk::holochain_json_api::{error::JsonError, json::JsonString};
use hdk::holochain_persistence_api::cas::content::Address;
use hdk::prelude::Entry;

pub mod receiver;
pub mod sender;

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct CounterpartySnapshot {
    balance: f64,
    valid: bool,
    invalid_reason: Option<String>,
    executable: bool,
    last_header_address: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct ChainSnapshot {
    pub snapshot: Vec<(ChainHeader, Entry)>,
}
