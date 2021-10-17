use crate::components::filter::{Filter, TransactionFilter, TransactionFilterOperation};
use crate::components::hero::Hero;
use crate::components::transaction_card::TransactionCard;
use crate::model::{Model, Msg, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::timeout::TimeoutService;
use yew::virtual_dom::{VList, VNode};
use yew::web_sys::Element;

pub mod components;
pub mod model;
pub mod string;
pub mod view_helpers;

const SECONDS_IN_DAY: u64 = 86400;

const BACKEND_URL: &str = "";
const FETCH_INTERVAL: u64 = 5;

const MOBILE_WIDTH: i32 = 768;
const NODE_PADDING: i32 = 2;
const ELEM_HEIGHT_DESKTOP: i32 = 150;
const ELEM_HEIGHT_MOBILE: i32 = 220;
const ELEM_MARGIN: i32 = 24;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct QueryParams {
    text_filter: Option<String>,
    filters: Option<Vec<TransactionFilter>>,
}

impl Default for QueryParams {
    fn default() -> Self {
        QueryParams {
            text_filter: None,
            filters: None,
        }
    }
}

fn current_timestamp() -> u64 {
    let current_date: js_sys::Date = js_sys::Date::new_0();
    let current_timestamp: f64 = current_date.get_time() / (1000_f64);
    current_timestamp as u64
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            first_fetch_done: false,
            transactions: vec![],
            loading: false,
            error: None,
            text_filter: Arc::new(None),
            feed_paused: false,
            transaction_filters: vec![],
            inclusion_filters: HashMap::new(),
            exclusion_filters: HashMap::new(),
            link,
            fetch_task: None,
            debounce_task: None,
            poll_task: None,
            animation_task: None,
            root_ref: NodeRef::default(),
            viewport_ref: NodeRef::default(),
            spacer_ref: NodeRef::default(),
            scroll_top: 0,
            root_height: 600,
            row_height: ELEM_HEIGHT_DESKTOP + ELEM_MARGIN,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchTransactions => {
                if self.feed_paused {
                    return false;
                }

                let after = self.transactions.first().map(|tx| tx.timestamp);
                let fetch_task = self.fetch_transactions(after);
                self.fetch_task = Some(fetch_task);
                self.loading = true;

                true
            }
            Msg::TransactionsFetched(data) => {
                self.loading = false;

                // Immediately show transactions as soon as the page loads.
                // For the subsequent loads, defer showing them to achieve an animation
                let new_transactions: Vec<Transaction> = data
                    .into_iter()
                    .map(|tx| Transaction {
                        animate: Some(self.first_fetch_done),
                        ..tx
                    })
                    .collect();

                if !self.first_fetch_done {
                    self.first_fetch_done = true;

                    self.transactions = new_transactions;
                } else {
                    // Add new elements at the head, and remove expired elements from tail
                    let now = current_timestamp();
                    let new_transactions_len = new_transactions.len();
                    self.transactions.splice(..0, new_transactions);
                    self.transactions.retain(|tx| now - tx.timestamp < SECONDS_IN_DAY);

                    // Remove animation for new transactions.
                    // This value needs to be kept in sync with the CSS
                    let cb = self.link.callback(move |_| Msg::RemoveAnimation(new_transactions_len));
                    let animation_task = TimeoutService::spawn(std::time::Duration::from_secs(1), cb);
                    self.animation_task = Some(animation_task);
                }

                // Poll for new data
                let cb = self.link.callback(|_| Msg::FetchTransactions);
                let poll_task = TimeoutService::spawn(std::time::Duration::from_secs(FETCH_INTERVAL), cb);
                self.poll_task = Some(poll_task);

                true
            }
            Msg::RemoveAnimation(n) => {
                if n == 0 {
                    return false;
                }

                for i in 0..self.transactions.len() - 1 {
                    if self.transactions[i].animate == Some(true) {
                        self.transactions[i].animate = Some(false);
                    }
                }

                true
            }
            Msg::HttpError(error) => {
                self.error = Some(error);
                self.loading = false;

                true
            }
            Msg::DebounceFilter(filter) => {
                self.debounce_task = None;

                let cb = self.link.callback(move |_| Msg::EditFilter(filter.to_owned()));
                let debounce_task = TimeoutService::spawn(std::time::Duration::from_millis(300), cb);
                self.debounce_task = Some(debounce_task);

                false
            }
            Msg::EditFilter(filter) => {
                if filter.trim().is_empty() {
                    self.text_filter = Arc::new(None);
                } else {
                    self.text_filter = Arc::new(Some(filter.trim().into()));
                }

                self.root_ref.cast::<Element>().unwrap().set_scroll_top(0);

                true
            }
            Msg::AddFilter(filter) => {
                self.transaction_filters.push(filter.clone());

                let map = match filter.operation {
                    TransactionFilterOperation::Include => &mut self.inclusion_filters,
                    TransactionFilterOperation::Exclude => &mut self.exclusion_filters,
                };

                match map.get_mut(&filter.text) {
                    Some(v) => v.push(filter),
                    None => {
                        map.insert(filter.text.clone(), vec![filter]);
                    }
                };

                true
            }
            Msg::RemoveFilter(filter) => {
                self.transaction_filters.retain(|f| f.ne(&filter));

                let map = match filter.operation {
                    TransactionFilterOperation::Include => &mut self.inclusion_filters,
                    TransactionFilterOperation::Exclude => &mut self.exclusion_filters,
                };

                // Remove entry
                if let Some(v) = map.get_mut(&filter.text) {
                    v.retain(|f| f.ne(&filter))
                }

                // If resulting vec is empty, remove the key/value pair from the map
                let len = map.get(&filter.text).map(|v| v.len()).unwrap_or(0);
                if len == 0 {
                    map.remove(&filter.text);
                }

                true
            }
            Msg::GenerateShareableUrl => {
                let text_filter = match (*self.text_filter).clone() {
                    Some(f) => Some(f),
                    None => None,
                };

                let filters = if self.transaction_filters.is_empty() {
                    None
                } else {
                    Some(self.transaction_filters.clone())
                };

                let params = QueryParams { text_filter, filters };
                let encoded = serde_qs::to_string(&params).unwrap();

                // yew::services::ConsoleService::log(format!("encoded: {:?}", encoded).as_str());
                let window = yew::utils::window();
                let url = format!("{}?{}", window.location().origin().unwrap(), encoded);
                window.location().set_href(&url).unwrap();

                false
            }
            Msg::ToggleFeedPaused => {
                self.feed_paused = !self.feed_paused;

                // Poll for new data
                let cb = self.link.callback(|_| Msg::FetchTransactions);
                let poll_task = TimeoutService::spawn(std::time::Duration::from_secs(FETCH_INTERVAL), cb);
                self.poll_task = Some(poll_task);

                true
            }
            Msg::OnScroll => {
                self.scroll_top = self.root_ref.cast::<Element>().unwrap().scroll_top();

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let now = current_timestamp();

        let oninput = self.link.callback(|event: InputData| Msg::DebounceFilter(event.value));

        let root_style = format!("height: {}px; overflow-y: auto", self.root_height);
        let viewport_style = format!(
            "overflow: hidden; height: {}px; position: relative",
            self.viewport_height()
        );
        let spacer_style = format!("transform: translateY({}px)", self.offset_y());

        let transactions = self.filtered_transactions();
        let min = self.start_index() as usize;
        let max = (self.start_index() + self.visible_items_count() - 1) as usize;

        let filter = match (*self.text_filter).clone() {
            Some(f) => f,
            None => "".to_string(),
        };

        html! {
            <>
            <Hero />

            <section class="section">
                <div class="container">
                    {self.view_loading()}

                    {self.view_error()}

                    <input class="input" type="search" placeholder="Search transactions" value={filter} oninput=oninput />
                    <button onclick={self.link.callback(|_| Msg::GenerateShareableUrl)}>
                        {"Share"}
                    </button>

                    <div class="filters">
                        <div class="field is-grouped is-grouped-multiline">
                            {for self.transaction_filters.iter().map(|f| html! {
                                <Filter
                                    filter={f.clone()}
                                    remove_filter={self.link.callback(Msg::RemoveFilter)} />
                            })}
                        </div>
                    </div>

                    <div class="settings">
                        <span class="transactions-description">
                            { format!{"{} transactions in the last 24 hours", transactions.len()} }
                        </span>
                        <label class="checkbox">
                            <input type="checkbox" onchange={self.link.callback(|_| Msg::ToggleFeedPaused)} checked={self.feed_paused} />
                            { view_helpers::space() }
                            { "Pause feed" }
                        </label>
                    </div>

                    <div class="root" ref=self.root_ref.clone() style=root_style onscroll={self.link.callback(|_| Msg::OnScroll)}>
                        <div class="viewport" ref=self.viewport_ref.clone() style=viewport_style>
                            <div class="spacer" ref=self.spacer_ref.clone() style=spacer_style>
                                {for transactions.iter().enumerate().filter(|&(i, _)| i >= min && i <= max).map(|(_, tx)| html! {
                                    <TransactionCard
                                        tx={tx.clone()}
                                        now={now}
                                        text_filter={self.text_filter.clone()}
                                        add_filter={self.link.callback(Msg::AddFilter)} />
                                })}
                            </div>
                        </div>
                    </div>
                </div>
            </section>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            // Init fixed row height based on window size
            let window = yew::utils::window();
            let width = window.inner_width().unwrap().as_f64().unwrap() as i32;
            if width <= MOBILE_WIDTH {
                self.row_height = ELEM_HEIGHT_MOBILE + ELEM_MARGIN;
            }

            // Check if query string contains some filters
            let mut query_string = window.location().search().unwrap_or("".to_string());
            if query_string.len() >= 1 {
                query_string.replace_range(0..1, "&");
            }
            let qs_config = serde_qs::Config::new(5, false);
            let params: QueryParams = qs_config.deserialize_str(&query_string).unwrap_or_default();
            yew::services::ConsoleService::log(format!("params: {:?}", params).as_str());

            if let Some(text_filter) = params.text_filter {
                self.link
                    .callback(move |_| Msg::EditFilter(text_filter.clone()))
                    .emit(());
            }

            if let Some(v) = params.filters {
                for filter in v {
                    self.link.callback(move |_| Msg::AddFilter(filter.clone())).emit(());
                }
            }

            // Fetch first batch of transactions
            let initial_fetch = self.link.callback(|_| Msg::FetchTransactions);
            initial_fetch.emit(());
        }
    }
}

impl Model {
    // Transactions
    fn fetch_transactions(&self, after: Option<u64>) -> FetchTask {
        let callback = self
            .link
            .callback(move |response: Response<Json<anyhow::Result<Vec<Transaction>>>>| {
                let (meta, Json(body)) = response.into_parts();

                match (meta.status.is_success(), body) {
                    (true, Ok(data)) => Msg::TransactionsFetched(data),
                    (false, Ok(_)) => Msg::HttpError(format!("Generic error, received {}", meta.status)),
                    (_, Err(error)) => Msg::HttpError(format!("{:?}", error)),
                }
            });

        let uri = match after {
            None => format!("{}{}", BACKEND_URL, "/transactions"),
            Some(a) => format!("{}{}?after={}", BACKEND_URL, "/transactions", a),
        };

        let request = Request::get(uri).body(Nothing).expect("Failed to build request");

        FetchService::fetch(request, callback).expect("Failed to start request")
    }

    fn in_inclusion_filters(&self, tx: &&Transaction) -> bool {
        self.inclusion_filters.values().all(|v| {
            v.iter()
                .any(|r| r.text == tx.from || r.text == tx.to || r.text == tx.message)
        })
    }

    fn in_exclusion_filters(&self, s: String) -> bool {
        self.exclusion_filters
            .get(&s)
            .map(|v| v.iter().any(|r| r.text == s))
            .unwrap_or(false)
    }

    fn filter_transaction(&self, tx: &&Transaction) -> bool {
        let mut keep = if self.inclusion_filters.keys().len() > 0 {
            self.in_inclusion_filters(tx)
        } else {
            true
        };

        if !keep {
            return false;
        }

        keep = if self.exclusion_filters.keys().len() > 0 {
            !(self.in_exclusion_filters(tx.message.clone())
                || self.in_exclusion_filters(tx.from.clone())
                || self.in_exclusion_filters(tx.to.clone()))
        } else {
            true
        };

        keep
    }

    fn filtered_transactions(&self) -> Vec<Transaction> {
        if let Some(ref f) = *self.text_filter {
            self.transactions
                .iter()
                .filter(|tx| tx.message.to_lowercase().contains(&f.to_lowercase()))
                .filter(|tx| self.filter_transaction(tx))
                .cloned()
                .collect()
        } else {
            self.transactions
                .iter()
                .filter(|tx| self.filter_transaction(tx))
                .cloned()
                .collect()
        }
    }

    // View
    fn view_loading(&self) -> Html {
        if self.loading && !self.first_fetch_done {
            html! {
               <div class="loader-wrapper is-active">
                   <div class="loader is-loading"></div>
               </div>
            }
        } else {
            VNode::from(VList::new())
        }
    }

    fn view_error(&self) -> Html {
        if self.error.is_some() {
            html! {
               <div>
                    <span>{ format!("{:?}", self.error) }</span>
               </div>
            }
        } else {
            VNode::from(VList::new())
        }
    }

    // Virtual scroll
    fn items_count(&self) -> i32 {
        self.filtered_transactions().len() as i32
    }

    fn viewport_height(&self) -> i32 {
        self.items_count() * self.row_height
    }

    fn start_index(&self) -> i32 {
        let start_node = (self.scroll_top / self.row_height) - NODE_PADDING;
        std::cmp::max(start_node, 0)
    }

    fn visible_items_count(&self) -> i32 {
        let count = (self.root_height / self.row_height) + 2 * NODE_PADDING;
        std::cmp::min(self.items_count() - self.start_index(), count)
    }

    fn offset_y(&self) -> i32 {
        self.start_index() * self.row_height
    }
}

fn main() {
    yew::start_app::<Model>();
}
