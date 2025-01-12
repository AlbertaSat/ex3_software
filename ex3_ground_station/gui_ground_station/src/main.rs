use yew::prelude::*;
use stylist::{yew::styled_component};

#[styled_component(SubSystemSelect)]
fn sub_system_select() -> Html {

    let active_ss: UseStateHandle<String> = use_state(|| "Ground Station".to_string());

    // Todo: Refactor so that the styling isn't inside the function
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
        
        let active_ss = active_ss.clone();
        
        Callback::from(move |item: String| {
            // Update the state based on the clicked item
            active_ss.set(item); // Update the state
        })
    };

    let sub_system_fullnames = vec!["Bulk Message Dispatcher", "UHF Subsystem", "Comms Handler", "Command Dispatcher", "Ground Station"];
    html! {
        <>
            <div>
                <h>{(*active_ss).clone()}</h>
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