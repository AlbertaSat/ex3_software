# C message dispatcher

This is a rudimentary implementation of the message dispatcher architectural component with the Ex-Alta3 Flight Software (FSW), in C.

## Architectural Description

 Right now the messge dispatcher is the first arch component to start following the OS boot. It can be thought of as the 'post office' of the FSW, and is responsible for routing messages within the On Board Computer (OBC), based on a 'destination ID' field within each message.

## Implementation

 This implementation uses the UNIX domain socket family, with SOCK_SEQPACKET type sockets to communicate data among FSW components.  
 It acts as a central server that  creates, binds to, and listens on a connection socket for incomming connection requests from clients - one for each arch component.
 It blocks upon start in a loop until all expected clients have formed a connection.
 Then it loops indefinitely, polling each connection for input on the main thread.
 It then reads the destination ID in the message, and sends the message recevied to the arch component with the associated ID.

## Usage

Use the 'run.sh' script with bash and gnome-terminal.

Use the following command to launch a server and specified number of clients:

```@sh
bash run.sh <num_clients>
```
