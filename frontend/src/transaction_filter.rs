#[derive(Clone, Debug)]
pub struct TransactionFilter {
    pub field: TransactionFilterField,
    pub operation: TransactionFilterOperation,
    pub text: String,
}

#[derive(Clone, Debug)]
pub enum TransactionFilterField {
    From,
    To,
    Message,
}

#[derive(Clone, Debug)]
pub enum TransactionFilterOperation {
    Include,
    Exclude,
}
