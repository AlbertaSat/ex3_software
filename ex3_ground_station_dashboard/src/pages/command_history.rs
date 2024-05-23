use yew::{function_component, html, Html};

use crate::components::navbar::Navbar;

#[function_component]
pub fn CommandHistory() -> Html {
    html! {
        <div>
            <Navbar/>
            <p>{"Command History"}</p>
        </div>
    }
}
