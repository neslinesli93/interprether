use yew::prelude::*;

pub struct Hero;

impl Component for Hero {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        unimplemented!()
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <section class="hero is-small">
                <div class="hero-head">
                    <nav class="navbar">
                        <div class="container">
                            <div class="navbar-end">
                                <span class="navbar-item">
                                    <a class="button is-small is-primary is-outlined" href="https://gitcoin.co/grants/3838/interprether" target="_blank" rel="noopener noreferrer">
                                        <span>{ "Gitcoin ➚" }</span>
                                    </a>
                                </span>
                                <span class="navbar-item">
                                    <a class="button is-small is-dark is-outlined" href="https://github.com/neslinesli93/interprether" target="_blank" rel="noopener noreferrer">
                                        <span class="icon">
                                            <i class="fab fa-github"></i>
                                        </span>
                                        <span>{ "Source ➚" }</span>
                                    </a>
                                </span>
                            </div>
                        </div>
                    </nav>
                </div>

                <div class="hero-body">
                    <div class="container has-text-centered">
                        <p class="title">{ "Interprether" }</p>
                        <p class="subtitle"> { "Real time feed of Ethereum transactions whose input data can be decoded in UTF-8" } </p>
                    </div>
                </div>
            </section>
        }
    }
}
