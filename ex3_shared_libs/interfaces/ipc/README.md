# IPC Interface

This is the libary used by various OBC FSW component to communicate with eachother using IPC Unix Domain Sockets of type SOCKSEQ packet.

The library provides a Server and Client struct, and helper functions to allow the Component using them to poll for incomming connection requests or data.

## Usage

Other FSW components can use this library by importing it in their Cargo.toml file, and using the ```new``` constructors for both Server and Client types to create an assocaited interface.

Client socket inputs are read using the ```poll_ipc_client_sockets``` function, which takes a vector of IpcClient objects.

Server socket inputs are read using the ```poll_ipc_sever_sockets``` function, which takes a vector of IpcServer objects.

## IMPORTANT

When data is read the associated buffer of that object is mutated, and thus it is **UP TO THE USER OF THE INTERFACE** to clear the buffer after they are done reading data from it, before they perform another read.
