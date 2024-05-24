mod pages;
mod components;
mod router;

use yew::{html, function_component, Html};
use yew_router::{BrowserRouter, Switch};

use router::{switch, Route};

#[function_component]
pub fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch}/>
        </BrowserRouter>
    }
}