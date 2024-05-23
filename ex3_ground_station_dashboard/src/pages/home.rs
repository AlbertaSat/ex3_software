use yew::{function_component, html, Html};

use crate::components::navbar::Navbar;

#[function_component]
pub fn Home() -> Html {
    html! {
        <div>
            <Navbar/>
            <p>{"home"}</p>
        </div>
    }
}
