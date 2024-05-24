use stylist::yew::styled_component;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[styled_component]
pub fn Navbar() -> Html {
    let css = css!(
        r#"
        .navbar {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1rem 2rem;
            background-color: #333;
            color: white;
        }
        .logo {
            display: flex;
            align-items: center;
        }
        .logo img {
            height: 40px;
            margin-right: 0.5rem;
        }
        .nav-links {
            list-style: none;
            display: flex;
            gap: 1rem;
        }
        .nav-links li {
            cursor: pointer;
        }
        .nav-links a {
            text-decoration: none;
            color: white;
            transition: color 0.3s ease;
        }
        .nav-links a:hover {
            text-decoration: underline;
        }
        .nav-links a:visited {
            color: white;
        }
        "#
    );
    
    html! {
        <div class={css}>
            <div class="navbar">
                <div class="logo">
                    {"logo"}
                </div>                
                <ul class="nav-links">
                    <li><Link<Route> to={Route::Home}>{ "Home" }</Link<Route>></li>
                    <li><Link<Route> to={Route::CommandHistory}>{ "Command History" }</Link<Route>></li>
                    <li><Link<Route> to={Route::SendCommand}>{ "Send Command" }</Link<Route>></li>
                </ul>
            </div>
        </div>
    }
}
