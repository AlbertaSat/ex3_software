use yew::{function_component, html, use_state, Html};
use web_sys::{HtmlInputElement, console};
use yew::events::KeyboardEvent;
use yew::TargetCast;

use crate::types::terminal_command::TerminalCommand;

#[function_component(TerminalPage)]
pub fn terminal_page() -> Html {
  let session_history = use_state(|| vec![]);
  let input = use_state(|| String::new());

  let onkeypress = {
    let session_history = session_history.clone();
    let input = input.clone();
    move |e: KeyboardEvent| {
      if e.key() == "Enter" {
        let value = (*input).clone();
        if !value.is_empty() {
          // Parse the input into a command and arguments
          let mut parts = value.split_whitespace();
          let name = parts.next().unwrap_or("").to_string();
          let arguments = parts.map(|s| s.to_string()).collect();

          // Create a TerminalCommand and execute it
          let command = TerminalCommand::new(&name, arguments);
          let output = command.execute();

          // Handle special case for clear command
          if name == "clear" {
            session_history.set(vec![]);
          } else {
            // Update session history
            session_history.set({
              let mut history = (*session_history).clone();
              history.push(format!("{}", value));
              history.push(output);
              history
            });
          }

          // Clear the input
          input.set(String::new());
        }
      }
    }
  };

  let oninput = {
    let input = input.clone();
    move |e: yew::events::InputEvent| {
      let input_element = e.target_unchecked_into::<HtmlInputElement>();
      let value = input_element.value();
      input.set(value);
    }
  };

  html! {
    <>
      <div class="bg-black w-screen p-2" style="height: 100vh">
        <div class="overflow-y-auto" style="width: 100%; font-size: 12px; font-family: monospace;">
          { for (*session_history).chunks(2).map(|chunk| html! {
            <div>
              <span style="color: #2AAA8A;">{"AlbertaSat/ExAlta3 $"}</span>
              <span class="text-white mx-2">{chunk[0].clone()}</span>
              <br/>
              <span class="text-white">{chunk[1].clone()}</span>
            </div>
          }) }
        </div>
        <div style="width: 100%; font-size: 12px; font-family: monospace;">
          <span style="color: #2AAA8A;">{"AlbertaSat/ExAlta3 $"}</span>
          <input
            class="bg-transparent text-white mx-2"
            style="outline: none; width: 85%;"
            type="text"
            value={(*input).clone()}
            oninput={oninput}
            onkeypress={onkeypress}
            autofocus=true
          />
        </div>
      </div>
    </>
  }
}