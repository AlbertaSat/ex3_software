use yew_router::Routable;

use crate::pages::home::Home;
use crate::pages::command_history::CommandHistory;
use crate::pages::send_command::SendCommand;
use yew::{ html, Html};


#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[at("/")]
        Home,
    #[at("/command_history")]
        CommandHistory,
    #[at("/send_command")]
        SendCommand
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html!{<Home/>},
        Route::CommandHistory => html!(<CommandHistory/>),
        Route::SendCommand => html!(<SendCommand/>)
    }
}