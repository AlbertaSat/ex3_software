use yew::{function_component, html, Html};

use crate::components::navbar::Navbar;

#[function_component]
pub fn SendCommand() -> Html {
    html! {
        <div>
            <Navbar/>
            <p>{"Send Command"}</p>
        </div>
    }
}
