use serde::Deserialize;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::virtual_dom::{VList, VNode};

const BACKEND_URL: &str = "http://localhost:3030";

#[derive(Deserialize, Debug)]
pub struct Transaction {
    pub hash: String,
    pub message: String,
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
                <div class="card-content">
                    <div class="content">
                        { &self.message }
                    </div>
                </div>
            </div>
        }
    }
}

enum Msg {
    FetchTransactions,
    TransactionsFetched(Vec<Transaction>),
    HttpError(String),
}

struct Model {
    transactions: Vec<Transaction>,
    loading: bool,
    error: Option<String>,
    link: ComponentLink<Self>,
    fetch_task: Option<FetchTask>,
}

impl Model {
    fn fetch_transactions(&mut self) -> FetchTask {
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
        let request = Request::get(format!("{}{}", BACKEND_URL, "/transactions"))
            .body(Nothing)
            .expect("Failed to build request");

        FetchService::fetch(request, callback).expect("Failed to start request")
    }

    fn view_loading(&self) -> Html {
        if self.loading {
            html! { "Loading..."}
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
            transactions: vec![],
            loading: false,
            error: None,
            link,
            fetch_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchTransactions => {
                let fetch_task = self.fetch_transactions();
                self.fetch_task = Some(fetch_task);
                self.loading = true;

                true
            }
            Msg::TransactionsFetched(data) => {
                self.transactions = data;
                self.loading = false;

                true
            }
            Msg::HttpError(error) => {
                self.error = Some(error);
                self.loading = false;

                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <section class="section">
                {self.view_loading()}

                {for self.transactions.iter().map(|tx| tx.render())}
            </section>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
