use crate::model::Transaction;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::sync::Arc;
use yew::classes;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tx: Transaction,
    pub now: u64,
    pub text_filter: Arc<Option<String>>,
    pub add_filter_message: Callback<String>,
    pub remove_filter_message: Callback<String>,
}

pub struct TransactionCard {
    props: Props,
}

impl Component for TransactionCard {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props != props {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let animate = match self.props.tx.animate {
            Some(true) => Some("animate"),
            _ => None,
        };

        let link = format!("https://etherscan.io/tx/{}", self.props.tx.hash);

        // Create human-readable time
        let duration = chrono::Duration::seconds(self.props.tx.timestamp as i64 - self.props.now as i64);
        let human_time = chrono_humanize::HumanTime::from(duration);

        // Create ISO time representation
        let naive = NaiveDateTime::from_timestamp(self.props.tx.timestamp as i64, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
        let iso_time = datetime.to_rfc2822();

        let message = self.props.tx.message.clone();
        let from = match &self.props.tx.from {
            Some(f) => f,
            None => "-",
        };
        let to = match &self.props.tx.to {
            Some(f) => f,
            None => "-",
        };

        html! {
            <div class=classes!("card", animate) key=self.props.tx.hash.clone()>
                <div class="card-header card-header-tx">
                    <p class="card-header-title">
                        <span>{ "Tx" }</span>
                        { crate::view::common::space() }
                        <span class="has-text-weight-normal tx-hash">{ &self.props.tx.hash }</span>
                        { crate::view::common::space() }
                        <span class="has-text-weight-normal is-size-7 tx-timestamp" title=iso_time>{ format!("({})", human_time) }</span>
                    </p>
                    <div class="card-header-filters">
                        <button
                            class="card-header-icon card-header-icon-filter"
                            title="Filter for message"
                            onclick={self.props.add_filter_message.reform(move |_| message.clone())}>
                                <i class="fas fa-search-plus" aria-hidden="true"></i>
                        </button>
                        <button class="card-header-icon card-header-icon-filter" title="Filter out message">
                            <i class="fas fa-search-minus" aria-hidden="true"></i>
                        </button>
                    </div>
                    <button class="card-header-icon" title="View on Etherscan">
                        <a href=link target="_blank" class="icon">
                            <i class="fas fa-external-link" aria-hidden="true"></i>
                        </a>
                    </button>
                </div>

                <div class="card-header is-size-7">
                    <div class="card-header-from">
                        <p class="card-header-title">
                            <span>{ "From" }</span>
                            { crate::view::common::space() }
                            <span class="has-text-weight-normal">{ from }</span>
                        </p>
                        <div class="card-header-filters pr-6">
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter for sender">
                                    <i class="fas fa-search-plus" aria-hidden="true"></i>
                            </button>
                            <button class="card-header-icon card-header-icon-filter" title="Filter out sender">
                                <i class="fas fa-search-minus" aria-hidden="true"></i>
                            </button>
                        </div>
                    </div>

                    <div class="card-header-to is-flex-grow-1">
                        <p class="card-header-title">
                            <span>{ "To" }</span>
                            { crate::view::common::space() }
                            <span class="has-text-weight-normal">{ to }</span>
                        </p>
                        <div class="card-header-filters">
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter for received">
                                    <i class="fas fa-search-plus" aria-hidden="true"></i>
                            </button>
                            <button class="card-header-icon card-header-icon-filter" title="Filter out received">
                                <i class="fas fa-search-minus" aria-hidden="true"></i>
                            </button>
                        </div>
                    </div>
                </div>

                <div>
                    <figure class="highlight">
                        <pre>
                            <code>{ self.render_message(self.props.text_filter.clone()) }</code>
                        </pre>
                    </figure>
                </div>
            </div>
        }
    }
}

impl TransactionCard {
    fn render_message(&self, filter: Arc<Option<String>>) -> Html {
        if let Some(f) = (*filter).clone() {
            let parts = crate::string::split_keep(&self.props.tx.message, &f);

            html! { {for parts.iter().map(|p| p.render())} }
        } else {
            html! { <span>{ &self.props.tx.message } </span> }
        }
    }
}
