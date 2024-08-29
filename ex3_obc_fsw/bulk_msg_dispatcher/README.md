# Bulk Message Dispatcher

The purpose of this dispatcher is to receive a Msg struct from a particular handler and read it's data from a provided path embedded in the body of the received Msg.

## Run

```cd``` into the ```bulk_msg_dispatcher``` directory in the ```ex3_obc_fsw``` directory and run:

```@sh
cargo run
```

## How it works

This process acts as a server for handling client connections to different payload handlers. It uses the IPC interface library that can be found in ```ex3_shared_libs```. When it receives a message it follows this protocol:

1. Determines the type of Msg it receives.
2. If it is a bulk Msg, it continues with the process below (else it sends it directly down to the GS handler).
3. It obtains the path from the body of the Msg it receives and builds a Msg with the large data to pass to the bulk_msg handling functions.
4. The functions mentioned above can be found in ```ex3_shared_libs```. The dispatcher uses them to slice the Msg into **4KB** packets.
5. After the vector of Msg's is ready, it executes a communication protocol with the GS handler for downlinking a large Msg. This protocol can be found [here](https://docs.google.com/document/d/18tPKPQxh9jXXWP5Zg5dk0lUnMpEQhH_QMLLM5OKqCLY/edit)

### Example Command

The CLI_GS expects a path to the data ending with a directory, a command could look like this:

```@sh
BulkMsgDispatcher ../handlers/dfgm_handler/dfgm_data
```

*As of now*, the bulk_msg_dispatcher reads all the files in the directory that it is passed. This can be configured to be certain files or parts of files in the future.
