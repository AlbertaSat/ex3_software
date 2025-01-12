use yew::prelude::*;
use stylist::{yew::styled_component};

#[styled_component(SubSystemSelect)]
fn sub_system_select() -> Html {
    let styling = css!(
        r#"
        ul {
            margin: 0;
            position: fixed;
            bottom: 0;
            list-style: none;
            width: 100%;
            height: 35px;
            background-color: #116E11;
            align-items: stretch;
        }
        li {
            display: inline-block;
            height:33px;
            background-color: #0ACf0A;
            margin-top: 0;
            margin-bottom: 0;
            margin-left: 2px;
            border: 1px solid green;
            width: 200px;
            align-items: center;
            text-align: center;
            padding: 7px 30px;
            cursor: pointer;
        }
        a {
            text-decoration: none;
            color: #000000
        }
        "#
    );

    enum SubSystem {
        BulkMessageDispatcher,
        UHFHandler,
        COMSHandler,
        ShellHandler,
        CommandDispatcher,
        GroundStation,
    }

    let _state = use_state(|| SubSystem::GroundStation);

    html! {
        <>
            <div>
                <h>{"Hello"}</h>
            </div>
            <div class={styling}>
                <ul>
                    <li ><a href="default.asp">{"Bulk Message Dispatcher"}</a></li>
                    <li ><a href="news.asp">{"UHF Subsystem"}</a></li>
                    <li><a href="contact.asp">{"Comms Handler"}</a></li>
                    <li><a href="about.asp">{"Command Dispatcher"}</a></li>
                    <li><a href="about.asp">{"Ground Station"}</a></li>
                </ul>
            </div>
        </>
    }
}

#[styled_component(App)]
fn app() -> Html {

    html! {
        <div>
            <SubSystemSelect/>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}