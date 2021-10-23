use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yew::virtual_dom::{VList, VNode};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TransactionFilter {
    pub field: TransactionFilterField,
    pub operation: TransactionFilterOperation,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum TransactionFilterField {
    From,
    To,
    Message,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum TransactionFilterOperation {
    Include,
    Exclude,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub filter: TransactionFilter,
    pub remove_filter: Callback<TransactionFilter>,
}

pub struct Filter {
    props: Props,
}

impl Component for Filter {
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
        let filter = self.props.filter.clone();

        html! {
            <div class="control">
                <div class="tags has-addons">
                    <span class="tag is-info">
                        { self.render_operation() }
                        { self.render_field() }
                    </span>
                    <a class="tag is-delete" onclick={self.props.remove_filter.reform(move |_| filter.clone())}></a>
                </div>
            </div>
        }
    }
}

impl Filter {
    fn render_operation(&self) -> Html {
        if self.props.filter.operation == TransactionFilterOperation::Exclude {
            html! {
                <>
                { "NOT" }
                { crate::view_helpers::space() }
                </>
            }
        } else {
            VNode::from(VList::new())
        }
    }

    fn render_field(&self) -> Html {
        let field = match self.props.filter.field {
            TransactionFilterField::From => "from",
            TransactionFilterField::To => "to",
            TransactionFilterField::Message => "message",
        };

        html! {
            { format!("{}:{}", field, self.props.filter.text) }
        }
    }
}
