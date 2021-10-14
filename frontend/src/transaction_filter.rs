#[derive(Clone, Debug)]
pub struct TransactionFilter {
    pub kind: TransactionFilterKind,
    pub text: String,
}

#[derive(Clone, Debug)]
pub enum TransactionFilterKind {
    From,
    To,
    Message,
}
