# DFGM Handler

To run: 

```@bash
cargo run
```

**Must have simulated DFGM running to read data**

It contains one interface for communication with the simulated DFGM over TCP and a second interface for Unix domain sockets that are used for internal communication. The TCP interface is created on the port specified in common::ports for the simulated environment. 

The handler will switch between reading and ignoring data that is sent to it each time an opcode of **0** is sent to it. This can be achieved using the cli_test_msg and specifying the opcode and dest_id of the msg.

