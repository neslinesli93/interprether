use crate::model::{Model, Msg, Transaction};
use crate::transaction_card::TransactionCard;
use std::sync::Arc;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::timeout::TimeoutService;
use yew::virtual_dom::{VList, VNode};
use yew::web_sys::Element;

pub mod model;
pub mod string;
pub mod transaction_card;
pub mod view;

const SECONDS_IN_DAY: u64 = 86400;

const BACKEND_URL: &str = "";
const FETCH_INTERVAL: u64 = 5;

const MOBILE_WIDTH: i32 = 768;
const NODE_PADDING: i32 = 2;
const ELEM_HEIGHT_DESKTOP: i32 = 150;
const ELEM_HEIGHT_MOBILE: i32 = 140;
const ELEM_MARGIN: i32 = 24;

fn current_timestamp() -> u64 {
    let current_date: js_sys::Date = js_sys::Date::new_0();
    let current_timestamp: f64 = current_date.get_time() / (1000_f64);
    current_timestamp as u64
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
        if let Some(ref f) = *self.filter {
            let filtered: Vec<Transaction> = self
                .transactions
                .iter()
                .filter(|tx| tx.message.to_lowercase().contains(&f.to_lowercase()))
                .cloned()
                .collect();

            filtered
        } else {
            self.transactions.to_owned()
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
                    { view::common::space() }
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
            filter: Arc::new(None),
            feed_paused: false,
            content_filters: vec![],
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
                    self.filter = Arc::new(None);
                } else {
                    self.filter = Arc::new(Some(filter.trim().into()));
                }

                self.root_ref.cast::<Element>().unwrap().set_scroll_top(0);

                true
            }
            Msg::AddMessageFilter(message) => {
                self.content_filters.push(message);
                true
            }
            Msg::RemoveMessageFilter(message) => {
                self.content_filters.retain(|f| f.eq(&message));
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
            <>
            {view::hero::render()}

            <section class="section">
                <div class="container">
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
                            { view::common::space() }
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
                                        text_filter={self.filter.clone()}
                                        add_filter_message={self.link.callback(|value| Msg::AddMessageFilter(value))}
                                        remove_filter_message={self.link.callback(|value| Msg::RemoveMessageFilter(value))} />
                                })
                                }
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
            let window = yew::utils::window();
            let width = window.inner_width().unwrap().as_f64().unwrap() as i32;
            if width <= MOBILE_WIDTH {
                self.row_height = ELEM_HEIGHT_MOBILE + ELEM_MARGIN;
            }

            let initial_fetch = self.link.callback(|_| Msg::FetchTransactions);
            initial_fetch.emit(());
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
