# CLI Ground Station

This is a cli version of the ground station that in the meantime is used for testing our code as we develop, and can be used in place of the GUI.
The goal is that operators can type the same (or similar) commands they would enter in the GUI and view the SC responses in the terminal directly.

## How it works

(As of now) The CLI GS runs a TCP server that awaits a client connection (from the COMS handler).

Once a connection is established the program loops, taking user input and writing it to the client via the TCP connection.

After an operator enters a message and it is sent to the COMS handler, a timer begins and the CLI GS waits for an ACK from the SC, to confirm the message was successfully delivered, or to become aware if the message was not delivered successfully. If the timer reaches its end then it is assumed the message was not delivered and  **TBD** ....  (ask operator to resend?)

An ACK can be:

- OK
- ERR (decryption failed)
- ERR (deserialization failed)

## Operator input format

Operators can input commands in the following format:

```@sh
<Subsystem/Payload name> <Op code number> <Data> 
```

Whereby, all values are seperated by a single blank space and:

- The first value is the name of the message destination
- The second value is the number of the desired opcode to send
- All following values are considered to be data - where the meaning of each value and expected order is dependent on the opcode

## Usage

The cli ground station can be run like any other cargo project, and takes no arguements:

```@sh
cargo run --bin cli_ground_station
```
