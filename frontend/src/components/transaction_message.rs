use crate::string::StringPartType;
use std::sync::Arc;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub message: String,
    pub filter: Arc<Option<String>>,
}

pub struct TransactionMessage {
    props: Props,
}

impl Component for TransactionMessage {
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
        if let Some(f) = (*self.props.filter).clone() {
            let parts = crate::string::split_keep(&self.props.message, &f);

            html! {
                {for parts.iter().map(|part| {
                    match part.t {
                        StringPartType::Normal => html! { <span> { &part.s } </span> },
                        StringPartType::Highlight => html! { <mark> { &part.s } </mark> },
                    }
                })}
            }
        } else {
            html! { <span>{ &self.props.message } </span> }
        }
    }
}
