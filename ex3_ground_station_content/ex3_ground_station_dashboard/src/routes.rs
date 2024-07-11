use yew::{ html, Html};
use yew_router::Routable;

use crate::pages::{home_page::HomePage, send_command_page::SendCommandPage};

#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[at("/")]
        Home,
    #[at("/send_command")]
        SendCommand
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html!{<HomePage />},
        Route::SendCommand => html!(<SendCommandPage />)
    }
}