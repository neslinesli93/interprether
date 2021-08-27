use serde::{Deserialize, Serialize};
use web3::types::H256;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    hash: String,
    message: String,
}

impl Transaction {
    pub fn new(hash: H256, message: &str) -> Self {
        Transaction {
            hash: format!("{:?}", hash),
            message: message.trim().into(),
        }
    }
}
