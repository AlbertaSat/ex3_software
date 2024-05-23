use yew::{function_component, html, Html};

use crate::components::navbar::Navbar;

#[function_component]
pub fn CommandList() -> Html {
    html! {
        <div>
            <Navbar/>
            <p>{"Command List"}</p>
        </div>
    }
}
