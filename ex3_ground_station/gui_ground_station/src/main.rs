use yew::prelude::*;
use stylist::yew::styled_component;

#[styled_component(App)]
fn app() -> Html {
    html! {
        <ul>
            <li><a href="default.asp">{"Bulk Message Dispatcher"}</a></li>
            <li><a href="news.asp">{"UHF Subsystem (simulated)"}</a></li>
            <li><a href="contact.asp">{"Comms Handler"}</a></li>
            <li><a href="about.asp">{"Command Dispatcher"}</a></li>
            <li><a href="about.asp">{"Ground Station"}</a></li>
        </ul>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}