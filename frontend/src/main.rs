use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use yew::classes;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::timeout::{TimeoutService, TimeoutTask};
use yew::virtual_dom::{VList, VNode};
use yew::web_sys::Element;

pub mod string;

const BACKEND_URL: &str = "http://localhost:3030";

const NODE_PADDING: i32 = 2;

const FETCH_INTERVAL: u64 = 5;

fn space() -> Html {
    html! { <span> { "\u{00a0}" }</span> }
}

fn current_timestamp() -> u64 {
    let current_date: js_sys::Date = js_sys::Date::new_0();
    let current_timestamp: f64 = current_date.get_time() / (1000 as f64);
    current_timestamp as u64
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

impl Transaction {
    fn render(&self, now: u64, filter: Option<&String>) -> Html {
        let animate = match self.animate {
            Some(true) => Some("animate"),
            _ => None,
        };

        let link = format!("https://etherscan.io/tx/{}", self.hash);

        // Create human-readable time
        let duration = chrono::Duration::seconds((self.timestamp - now) as i64);
        let human_time = chrono_humanize::HumanTime::from(duration);

        // Create ISO time representation
        let naive = NaiveDateTime::from_timestamp(self.timestamp as i64, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let iso_time = datetime.to_rfc2822();

        html! {
            <div class=classes!("card", animate) key=self.hash.clone()>
                <header class="card-header">
                    <p class="card-header-title">
                        <span>{ "Tx" }</span>
                        { space() }
                        <span class="has-text-weight-normal tx-hash">{ &self.hash }</span>
                        { space() }
                        <span class="has-text-weight-normal is-size-7 tx-timestamp" title=iso_time>{ format!("({})", human_time) }</span>
                    </p>
                    <button class="card-header-icon" aria-label="more options">
                        <a href=link target="_blank" class="icon">
                            <i class="fas fa-external-link" aria-hidden="true"></i>
                        </a>
                    </button>
                </header>
                <div>
                    <figure class="highlight">
                        <pre>
                            <code>{ self.render_message(filter) }</code>
                        </pre>
                    </figure>
                </div>
            </div>
        }
    }

    fn render_message(&self, filter: Option<&String>) -> Html {
        match filter {
            None => {
                html! { <span>{ &self.message } </span> }
            }
            Some(f) => {
                let parts = string::split_keep(&self.message, f);

                html! { {for parts.iter().map(|p| p.render())} }
            }
        }
    }
}

enum Msg {
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

struct Model {
    // Model
    first_fetch_done: bool,
    transactions: Vec<Transaction>,
    loading: bool,
    error: Option<String>,
    filter: Option<String>,
    feed_paused: bool,
    // Cmd bus
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    debounce_task: Option<TimeoutTask>,
    poll_task: Option<TimeoutTask>,
    animation_task: Option<TimeoutTask>,
    // Refs
    root_ref: NodeRef,
    viewport_ref: NodeRef,
    spacer_ref: NodeRef,
    scroll_top: i32,
    root_height: i32,
    row_height: i32,
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

    fn filtered_transactions(&self) -> Vec<Transaction> {
        match &self.filter {
            Some(f) => {
                let filtered: Vec<Transaction> = self
                    .transactions
                    .iter()
                    .filter(|tx| tx.message.to_lowercase().contains(&f.to_lowercase()))
                    .cloned()
                    .collect();

                filtered
            }
            None => self.transactions.to_owned(),
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

    fn view_filter_description(&self) -> Html {
        if self.filter.is_some() {
            html! {
                <>
                    { space() }
                    <span> { "(w/ filter)" } </span>
                </>
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

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            first_fetch_done: false,
            transactions: vec![],
            loading: false,
            error: None,
            filter: None,
            feed_paused: false,
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
            row_height: 100 + 24,
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
                    self.transactions.retain(|tx| now - tx.timestamp < 86400);

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
                self.error = Some(error.clone());
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
                    self.filter = None;
                } else {
                    self.filter = Some(filter.trim().into())
                }

                self.root_ref.cast::<Element>().unwrap().set_scroll_top(0);

                true
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

        html! {
            <div class="container">
                <section class="section">
                    {self.view_loading()}

                    {self.view_error()}

                    <input class="input" type="search" placeholder="Search transactions" oninput=oninput />

                    <div class="settings">
                        <span class="transactions-description">
                            { format!{"{} transactions in the last 24 hours", transactions.len()} }
                            { self.view_filter_description() }
                        </span>
                        <label class="checkbox">
                            <input type="checkbox" onchange={self.link.callback(|_| Msg::ToggleFeedPaused)} checked={self.feed_paused} />
                            { space() }
                            { "Pause feed" }
                        </label>
                    </div>

                    <div class="root" ref=self.root_ref.clone() style=root_style onscroll={self.link.callback(|_| Msg::OnScroll)}>
                        <div class="viewport" ref=self.viewport_ref.clone() style=viewport_style>
                            <div class="spacer" ref=self.spacer_ref.clone() style=spacer_style>
                                {for transactions
                                    .iter()
                                    .enumerate()
                                    .filter(|&(i, _)| i >= min && i <= max)
                                    .map(|(_, tx)| tx.render(now, self.filter.as_ref()))
                                }
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            let initial_fetch = self.link.callback(|_| Msg::FetchTransactions);
            initial_fetch.emit(());
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
