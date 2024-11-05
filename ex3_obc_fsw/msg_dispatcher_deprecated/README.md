# Message Dispatcher in C (deprecated)

This is a rudimentary implementation of the message dispatcher architectural component with the Ex-Alta3 Flight Software (FSW), in C.

## Architectural Description

 Right now the messge dispatcher is among the first arch components to start following the OS boot. It can be thought of as the 'post office' of the FSW, and is responsible for routing messages within the On Board Computer (OBC), based on a 'destination ID' field within each message.

 The message dispatcher acts as a central server within the FSW, facilitating communication between other architectural components.

## Implementation / Design

The message dispatcher acts a central server, setting up and listening to unix domain sockets to establish connections with other architectural components in the FSW. Polling is used so that various sockets may communicate in a non-blocking fashion in the main process thread.

It polls on a 'conn socket' file descriptor awaiting a client connection request, and upon accepting the request, then polls the 'data socket' file descriptor to read incoming data.

Upon accepting a client connection request, the 'connected' flag of the component struct associated with that client is set to one. Whenever a read of the connection returns 0, it means the client connection has been dropped, and the 'connected' flag is set back to zero, and the associated polling struct fd is set back to its connection fd.

## Usage

To run th message dispatcher first compile the source code using:

```@sh
make
```

Then run the resulting executable as

```@sh
./message_dispatcher
```

(For now ) The server sets up a socket and listens for an incomming connection based on a hard coded array of pointers to ComponentStruct structs. The name of component (i.e. dfgm_handler) are provided as strings, along with the assocaited ID of that component found [here](https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0).

The name of the component is in the path of the file created by the server (i.e. /tmp/fifo_dfgm_handler), and is used by the client upon sending a connection request so it knows what socket to connect to.

## Notes

The server code functions in the following order to establish connection with a client.

  1. Create connection (unix) sockets
  2. Bind the conn socket fd [from socket() call] to an address in unix domain (fifo file)
  3. Call listen on the bound socket for incomming conn requests from clients
  4. Call accept to accept conn request from client - returns new descriptor for data between client and server
  5. Handle connection now with data socket
