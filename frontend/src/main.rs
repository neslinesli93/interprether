use serde::Deserialize;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::timeout::{TimeoutService, TimeoutTask};
use yew::virtual_dom::{VList, VNode};

const BACKEND_URL: &str = "http://localhost:3030";

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "h")]
    pub hash: String,
    #[serde(rename = "m")]
    pub message: String,
    #[serde(rename = "t")]
    pub timestamp: u64,
}

impl Transaction {
    fn render(&self) -> Html {
        let link = format!("https://etherscan.io/tx/{}", self.hash);

        html! {
            <div class="card" key=self.hash.clone()>
                <header class="card-header">
                    <p class="card-header-title">
                        <span>{ "Tx\u{00a0}" }</span>
                        <span class="has-text-weight-normal tx-hash">{ &self.hash }</span>
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
        if self.loading {
            html! { <div>{ "Loading..." }</div> }
        } else {
            VNode::from(VList::new())
        }
    }

    fn view_error(&self) -> Html {
        match &self.error {
            Some(e) => html! { <div>{ format!("Error: {:?}", e) }</div> },
            None => VNode::from(VList::new()),
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
                self.transactions.splice(..0, data);
                self.loading = false;

                let cb = self.link.callback(move |_| Msg::FetchTransactions);
                let poll_task = TimeoutService::spawn(std::time::Duration::from_secs(5), cb);
                self.poll_task = Some(poll_task);

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

        html! {
            <div class="container">
                <section class="section">
                    // TODO: Make these two better...
                    {self.view_loading()}
                    {self.view_error()}

                    <input class="input" type="search" placeholder="Search transactions" oninput=oninput />

                    {for self.transactions.iter().filter(filter).map(|tx| tx.render())}
                </section>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
