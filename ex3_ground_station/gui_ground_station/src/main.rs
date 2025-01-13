use yew::prelude::*;
use stylist::{yew::styled_component};

#[derive(Clone, PartialEq)]
enum SubSystem {
    BulkMessageDispatcher,
    UHFHandler,
    COMSHandler,
    ShellHandler,
    CommandDispatcher,
    GroundStation,
}

fn view_pager(ss: SubSystem) -> Html {
    match ss {
        SubSystem::BulkMessageDispatcher => html!{<h>{"BulkMessageDispatcher"}</h>},
        SubSystem::UHFHandler => html!{<h>{"UHFHandler"}</h>},
        SubSystem::COMSHandler => html!{<CommsTerminal/>},
        SubSystem::ShellHandler => html!{<h>{"ShellHandler"}</h>},
        SubSystem::CommandDispatcher => html!{<h>{"CommandDispatcher"}</h>},
        SubSystem::GroundStation => html!{<h>{"GroundStation"}</h>},
    }
}

#[styled_component(SubSystemSelect)]
fn sub_system_select() -> Html {

    let active_ss = use_state(|| SubSystem::GroundStation);

    // Todo: Refactor so that the styling isn't inside this function
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
    
    let onclick = {
        
        let activeSS = active_ss.clone();
        
        Callback::from(move |item: String| {
            // Update the state based on the clicked item
            let new_ss = match item.as_str() {
                "Bulk Message Dispatcher" => SubSystem::BulkMessageDispatcher,
                "UHF Subsystem" => SubSystem::UHFHandler,
                "Comms Handler" => SubSystem::COMSHandler,
                "Shell Handler" => SubSystem::ShellHandler,
                "Command Dispatcher" => SubSystem::CommandDispatcher,
                "Ground Station" => SubSystem::GroundStation,
                _ => SubSystem::GroundStation,
            };
            activeSS.set(new_ss); // Update the state
        })
    };

    let sub_system_fullnames = vec!["Bulk Message Dispatcher", "UHF Subsystem", "Comms Handler", "Shell Handler", "Command Dispatcher", "Ground Station"];
    html! {
        <>
            <div>
                <h>{view_pager((*active_ss).clone())}</h>
            </div>
            <div class={styling}>
                <ul>
                    { for sub_system_fullnames.iter().map(|item| {
                        let item_clone = item.to_string();
                        let onclick = {
                            let onclick = onclick.clone();
                            Callback::from(move |_| onclick.emit(item_clone.clone()))
                        };
                        html! {
                            <li onclick={onclick}>
                                {item}
                            </li>
                        }
                    })}
                </ul>
            </div>
        </>
    }
}

#[styled_component(CommsTerminal)]
fn coms_terminal_comp() -> Html {

    let styling = css!(
        r#"
        h {
            font-size: 25px;
            width: 100%;
        }

        #stateView {
            display: inline-block;
            width: 73%;
            height: 1000px;
            background-color: #eeeeee;
            font-size: 15px;
            border-color: #afafaf;
            border-size: 1px;
        }

        #opCodeView {
           display: inline-block;
            width: 23%;
            height: 1000px;
            background-color: #bebebe;
            font-size: 15px;
            border-color: #afafaf;
            border-size: 1px;
            padding: 5px;
        }

        #stateAndOpCodes {
            font-size: 0;
            display: flex;
        }

        #button {
            display: inline-block;
            background-color: #9e9e9e;
            padding: 5px;
            cursor: pointer;
        }
        
        "#
    );

    html! {
        <div class={styling}>
            <h>
                {"View Subsystem: Comms Handler"}
                <div id="stateAndOpCodes">
                    <p id="stateView">{"lorem ipsum lkajdjajdasdjsaldjs"}</p>
                    <p id="opCodeView">
                        <p id="button">{"Get Housekeeping"}</p>
                    </p>
                </div>
            </h>
        </div>
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