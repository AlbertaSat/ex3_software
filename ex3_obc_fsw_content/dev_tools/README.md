# Dev tools

This directory contains various little programs that we have created as we implement our software that allow us to quickly test the code we have written.

For example we have created mock TCP and IPC client/servers to inject data into components and observe behaviour and test our implementation.

## IPC Burst Hardcoded

This creates an IPC client, connects to the msg dispatcher as the component defined by the user (arg 1), writes a hardcoded msg to the msg dispatcher, and closes.

This is useful for quick and dirty sending IPC commands to a component from the message dispatcher.
