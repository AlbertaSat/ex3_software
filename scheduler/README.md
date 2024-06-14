# Scheduler
This scheduler reads a stream of bytes using a TCP port and deserializes it into a Msg Struct which can be found in libs/message_structure.

### Running
```bash
cargo run --bin scheduler
```
This creates a TCP server for clients to connect to.

Next, you can test its ability to read in a Msg struct by running the **cli_test_msg** directory which creates a client for the scheduler to read.

### Additional Info

The scheduler will also make two new directories when it runs:

- A *scheduler_logs* folder will contain logs that are made as part of the code.
- A *saved_commands* folder which will have the command saved in it's own timestamped file. As of now, if multiple commands are given, a new file will be created for each one. There is a rolling file system in place for this that holds 2 KB of files.

There is a script in the scripts directory to delete these extra directories if needed.