use serde::Deserialize;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::services::timeout::TimeoutTask;

pub enum Msg {
    // Transactions
    FetchTransactions,
    TransactionsFetched(Vec<Transaction>),
    RemoveAnimation(usize),
    HttpError(String),
    // Filter
    DebounceFilter(String),
    EditFilter(String),
    // Toggle
    ToggleFeedPaused,
    // Virtual scroll
    OnScroll,
}

pub struct Model {
    // Model
    pub first_fetch_done: bool,
    pub transactions: Vec<Transaction>,
    pub loading: bool,
    pub error: Option<String>,
    pub filter: Option<String>,
    pub feed_paused: bool,
    // Cmd bus
    pub link: ComponentLink<Self>,
    pub fetch_task: Option<FetchTask>,
    pub debounce_task: Option<TimeoutTask>,
    pub poll_task: Option<TimeoutTask>,
    pub animation_task: Option<TimeoutTask>,
    // Refs
    pub root_ref: NodeRef,
    pub viewport_ref: NodeRef,
    pub spacer_ref: NodeRef,
    pub scroll_top: i32,
    pub root_height: i32,
    pub row_height: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Transaction {
    // Backend fields
    #[serde(rename = "h")]
    pub hash: String,
    #[serde(rename = "m")]
    pub message: String,
    #[serde(rename = "t")]
    pub timestamp: u64,
    // Local model
    pub animate: Option<bool>,
}
