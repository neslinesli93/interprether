use serde::{Deserialize, Serialize};
use web3::types::H256;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    #[serde(rename = "h")]
    pub hash: String,
    #[serde(rename = "m")]
    pub message: String,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(hash: H256, message: String, timestamp: u64) -> Self {
        Transaction {
            hash: format!("{:?}", hash),
            message,
            timestamp,
        }
    }
}
