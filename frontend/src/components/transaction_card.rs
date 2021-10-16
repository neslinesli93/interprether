use crate::components::filter::{TransactionFilter, TransactionFilterField, TransactionFilterOperation};
use crate::components::transaction_message::TransactionMessage;
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
    pub add_inclusion_filter: Callback<TransactionFilter>,
    pub add_exclusion_filter: Callback<TransactionFilter>,
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
        let message_copy = self.props.tx.message.clone();

        let from = self.props.tx.from.clone();
        let from_copy = from.clone();

        let to = self.props.tx.to.clone();
        let to_copy = to.clone();

        html! {
            <div class=classes!("card", animate) key=self.props.tx.hash.clone()>
                <div class="card-header card-header-tx">
                    <p class="card-header-title">
                        <span>{ "Tx" }</span>
                        { crate::view_helpers::space() }
                        <span class="has-text-weight-normal tx-hash">{ &self.props.tx.hash }</span>
                        { crate::view_helpers::space() }
                        <span class="has-text-weight-normal is-size-7 tx-timestamp" title=iso_time>{ format!("({})", human_time) }</span>
                    </p>
                    <div class="card-header-filters">
                        <button
                            class="card-header-icon card-header-icon-filter"
                            title="Filter for message"
                            onclick={self.props.add_inclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::Message, operation: TransactionFilterOperation::Include, text: message.clone()})}>
                                <i class="fas fa-search-plus" aria-hidden="true"></i>
                        </button>
                        <button
                            class="card-header-icon card-header-icon-filter"
                            title="Filter out message"
                            onclick={self.props.add_exclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::Message, operation: TransactionFilterOperation::Exclude, text: message_copy.clone()})}>
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
                            { crate::view_helpers::space() }
                            <span class="has-text-weight-normal">{ from.clone() }</span>
                        </p>
                        <div class="card-header-filters pr-6">
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter for sender"
                                onclick={self.props.add_inclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::From, operation: TransactionFilterOperation::Include, text: from.clone()})}>
                                    <i class="fas fa-search-plus" aria-hidden="true"></i>
                            </button>
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter out sender"
                                onclick={self.props.add_exclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::From, operation: TransactionFilterOperation::Exclude, text: from_copy.clone()})}>
                                <i class="fas fa-search-minus" aria-hidden="true"></i>
                            </button>
                        </div>
                    </div>

                    <div class="card-header-to is-flex-grow-1">
                        <p class="card-header-title">
                            <span>{ "To" }</span>
                            { crate::view_helpers::space() }
                            <span class="has-text-weight-normal">{ to.clone() }</span>
                        </p>
                        <div class="card-header-filters">
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter for receiver"
                                onclick={self.props.add_inclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::To, operation: TransactionFilterOperation::Include, text: to.clone()})}>
                                    <i class="fas fa-search-plus" aria-hidden="true"></i>
                            </button>
                            <button
                                class="card-header-icon card-header-icon-filter"
                                title="Filter out receiver"
                                onclick={self.props.add_exclusion_filter.reform(move |_| TransactionFilter{field: TransactionFilterField::To, operation: TransactionFilterOperation::Exclude, text: to_copy.clone()})}>
                                <i class="fas fa-search-minus" aria-hidden="true"></i>
                            </button>
                        </div>
                    </div>
                </div>

                <div>
                    <figure class="highlight">
                        <pre>
                            <code>
                                <TransactionMessage
                                    message={self.props.tx.message.clone()}
                                    filter={self.props.text_filter.clone()} />
                            </code>
                        </pre>
                    </figure>
                </div>
            </div>
        }
    }
}
