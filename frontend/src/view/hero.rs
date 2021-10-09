use yew::prelude::*;

pub fn render() -> Html {
    html! {
        <section class="hero is-small">
            <div class="hero-head">
                <nav class="navbar">
                    <div class="container">
                        <div class="navbar-end">
                            <span class="navbar-item">
                                <a class="button" href="https://github.com/neslinesli93/interprether" target="_blank" rel="noopener noreferrer">
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
