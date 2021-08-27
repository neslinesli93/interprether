use serde::{Deserialize, Serialize};
use web3::types::H256;

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
    pub message: String,
}

impl Transaction {
    pub fn new(hash: H256, message: &str) -> Self {
        Transaction {
            hash: format!("{:?}", hash),
            message: message.trim().into(),
        }
    }
}
