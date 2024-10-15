# Coms Handler

The coms handler is the first and last part of the OBC FSW (aside from the interface that facilitates physical communication with the transceiver) for data communicated with the ground station (downlink and uplink).

## Scope of the Coms handler

It is responsible for the following things:

- Listening for incoming data from the UHF transceiver
- Listening to the IPC port for data incoming from other FSW components (either to talk to the handler and UHF directly, or to be downlinked using the UHF transcevier)
- Decrypting incomming messages. (All uplinked messages will be encryped in accordance with CSA requirements)
- Bulk message handling
  - Fragmenting large outgoing messages into chunks that fit within the message unit size (paylod of Ax.25 packet) defined by the UHF transceiver. ***As of now this is 128 bytes***
  - Rebuilding a large incomming message from its constiuent chunks (which as of now will be 128 byte chunks)

## How it works

The coms handler starts, creates an interface for communication with the OBC FSW (IPC), and the UHF Transceiver (for now Tcp to simulated uhf transceiver).
Then it enters a loop, polling both reading for input from either. If either has read some data (read returns more than 0 bytes), then a handler function takes the data and determines where to send it based on its associated header destination.

### When reading from IPC -> If we received something

- Deserialize the bytes (create message obj from bytes)
- Check message destination
  - Not for coms hanlder direclty, then
    - Check if the message needs to fragmented
      - If not bulk msg: write it to UHF transceiver (downlink it)
      - If bulk msg: 'handle it as a bulk msg' -> then
  - For coms handler directly, then handle it based on op code

### When reading from UHF -> If we have received something

- Emit an 'ack' that tells sender we got something
- Decrypt bytes
- Deserialize the bytes (create message obj from bytes)
- Check the message destination
  - if it is not for the coms handler directly, then forward it to the message dispatcher (write to IPC connection to message dispatcher)
  - If it is for the coms handler directly, then handle it based on op code

## Usage

First this component requires the msg dispacher to be running (IPC server awaiting client conn request), and the simulated UHF subsystem must be running (TCP server awaiting client conn request).

After these are running you can use the following command to stard the coms handler from the main cargo workspace directory:

```@sh
cargo run --bin coms_handler
```

Handlers should be able to be started in any order as they generate client requests when their associated process starts - so long as the servers are awaiting the client connection request it should work.

## Notes
