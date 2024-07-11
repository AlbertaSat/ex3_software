# DFGM Handler

To run: 

```@bash
cargo run
```

**Must have simulated DFGM running to read data**

It contains one interface for communication with the simulated DFGM over TCP and a second interface for Unix domain sockets that are used for internal communication. The TCP interface is created on the port specified in common::ports for the simulated environment. 

The handler will switch between reading and ignoring data that is sent to it each time an opcode of **0** is sent to it. This can be achieved using the cli_test_msg and specifying the opcode and dest_id of the msg.

### Run and Testing

1. Run the Msg Dispatcher, ```./msg_dispatcher```, after running ```make``` in the msg_dispatcher directory.
2. Run the simulated DFGM, ```python3 dfgm_subsystem.py``` in the ex3_simulated_subsystems repo.
3. Run the coms_handler, (this version of the DFGM handler used the ipc_dummy_client as a stand-in coms_handler running ```cargo run <port> coms_handler```)
4. ```cargo run``` in the dfgm_handler directory
5. Send a command to toggle the data collection using the cli_test_msg 