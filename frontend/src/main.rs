use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use yew::classes;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::timeout::{TimeoutService, TimeoutTask};
use yew::virtual_dom::{VList, VNode};

const BACKEND_URL: &str = "http://localhost:3030";

fn space() -> Html {
    html! { <span> { "\u{00a0}" }</span> }
}

#[derive(Deserialize, Debug)]
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
    fn render(&self, current_timestamp: u64) -> Html {
        let animate = match self.animate {
            Some(true) => Some("animate"),
            _ => None,
        };

        let link = format!("https://etherscan.io/tx/{}", self.hash);

        // Create human-readable time
        let duration = chrono::Duration::seconds((self.timestamp - current_timestamp) as i64);
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
                            <code>{ &self.message }</code>
                        </pre>
                    </figure>
                </div>
            </div>
        }
    }
}

enum Msg {
    FetchTransactions,
    TransactionsFetched(Vec<Transaction>),
    HttpError(String),
    DebounceFilter(String),
    EditFilter(String),
}

struct Model {
    first_fetch_done: bool,
    transactions: Vec<Transaction>,
    loading: bool,
    error: Option<String>,
    filter: Option<String>,
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
    debounce_task: Option<TimeoutTask>,
    poll_task: Option<TimeoutTask>,
}

impl Model {
    fn fetch_transactions(&mut self, after: Option<u64>) -> FetchTask {
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
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let initial_fetch = link.callback(|_| Msg::FetchTransactions);
        initial_fetch.emit(());

        Self {
            first_fetch_done: false,
            transactions: vec![],
            loading: false,
            error: None,
            filter: None,
            link,
            fetch_task: None,
            debounce_task: None,
            poll_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchTransactions => {
                let after = self.transactions.first().map(|tx| tx.timestamp);
                let fetch_task = self.fetch_transactions(after);
                self.fetch_task = Some(fetch_task);
                self.loading = true;

                true
            }
            Msg::TransactionsFetched(data) => {
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
                    // Add new elements at the head, and remove same amount from tail
                    for i in 0..self.transactions.len() - 1 {
                        if self.transactions[i].animate == Some(true) {
                            self.transactions[i].animate = Some(false);
                        }
                    }

                    let new_elements = new_transactions.len();
                    self.transactions.splice(..0, new_transactions);
                    self.transactions.truncate(self.transactions.len() - new_elements);
                }

                let cb = self.link.callback(move |_| Msg::FetchTransactions);
                let poll_task = TimeoutService::spawn(std::time::Duration::from_secs(15), cb);
                self.poll_task = Some(poll_task);

                self.loading = false;

                true
            }
            Msg::HttpError(error) => {
                self.error = Some(error.clone());
                self.loading = false;

                ConsoleService::log(format!("Error while fetching data: {}", error).as_str());

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

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let oninput = self.link.callback(|event: InputData| Msg::DebounceFilter(event.value));

        let filter = |tx: &&Transaction| {
            self.filter
                .as_ref()
                .map(|f| tx.message.to_lowercase().contains(&f.to_lowercase()))
                .unwrap_or(true)
        };

        let current_date: js_sys::Date = js_sys::Date::new_0();
        let current_timestamp: f64 = current_date.get_time() / (1000 as f64);
        let current_timestamp_trunc: u64 = current_timestamp as u64;

        html! {
            <div class="container">
                <section class="section">
                    {self.view_loading()}

                    {self.view_error()}

                    <input class="input" type="search" placeholder="Search transactions" oninput=oninput />

                    {for self.transactions.iter().filter(filter).map(|tx| tx.render(current_timestamp_trunc))}
                </section>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
