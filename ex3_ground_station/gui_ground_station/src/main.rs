use yew::prelude::*;
use stylist::{yew::styled_component};

#[styled_component(App)]
fn app() -> Html {
    let styling = css!(
        r#"
        ul {
            position: fixed;
            bottom: 0;
            list-style: none;
            width: 100%;
            height: 35px;
            background-color:rgb(17, 110, 17);
        }
        li {
            display: inline-block;
            align-items: stretch;
            background-color: #00ff00;
            padding: 5px;
            border: 1px solid green;
            width: 200px;
        }
        a {
            text-decoration: none;
            color: #000000
        }
        "#
    );

    html! {
        <div class={styling}>
            <ul>
                <li ><a href="default.asp">{"Bulk Message Dispatcher"}</a></li>
                <li ><a href="news.asp">{"UHF Subsystem (simulated)"}</a></li>
                <li><a href="contact.asp">{"Comms Handler"}</a></li>
                <li><a href="about.asp">{"Command Dispatcher"}</a></li>
                <li><a href="about.asp">{"Ground Station"}</a></li>
            </ul>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}