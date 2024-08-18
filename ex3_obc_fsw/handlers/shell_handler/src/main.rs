/*
Written by Kaaden RumanCam
Summer 2024

...

TODO
*/

//use common::{opcodes, ports};
use ipc::{IpcClient, poll_ipc_clients};

struct ShellHandler {
    msg_dispatcher_interface: IpcClient, // To get messages from the dispatcher
}

impl ShellHandler {
    pub fn new(
        msg_dispatcher_interface: IpcClient,
    ) -> ShellHandler {
        ShellHandler {
            msg_dispatcher_interface,
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        // Poll for messages
        loop {
            let mut clients = vec![
                &mut self.msg_dispatcher_interface,
            ];
            poll_ipc_clients(&mut clients)?;


        }
    }
}

fn main() {
    println!("Beginning Shell Handler...");

    let msg_dispatcher_interface = match IpcClient::new("shell_handler".to_string()) {
        Ok(mdi) => mdi,
        Err(e) => panic!("Error creating dispatcher interface: {e:?}"),
    };

    let mut shell_handler = ShellHandler::new(msg_dispatcher_interface);

    let _ = shell_handler.run();
}

