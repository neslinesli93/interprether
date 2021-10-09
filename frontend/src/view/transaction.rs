use crate::model::Transaction;
use chrono::{DateTime, NaiveDateTime, Utc};
use yew::classes;
use yew::prelude::*;

impl Transaction {
    pub fn render(&self, now: u64, filter: Option<&String>) -> Html {
        let animate = match self.animate {
            Some(true) => Some("animate"),
            _ => None,
        };

        let link = format!("https://etherscan.io/tx/{}", self.hash);

        // Create human-readable time
        let duration = chrono::Duration::seconds(self.timestamp as i64 - now as i64);
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
                        { crate::view::common::space() }
                        <span class="has-text-weight-normal tx-hash">{ &self.hash }</span>
                        { crate::view::common::space() }
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
                let parts = crate::string::split_keep(&self.message, f);

                html! { {for parts.iter().map(|p| p.render())} }
            }
        }
    }
}
