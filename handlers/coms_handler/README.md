# Coms Handler

The coms handler is the first and last part of the OBC FSW (aside from the interface the faciliates phsyical communication with the transceiver) for data communicated with the ground station (downlink and uplink).

## Scope of the Coms handler

It is responsible for the following things:

- Listening for incoming data from the UHF transceiver
- Listening to the IPC port for data incoming from other FSW components (either to talk to the handler and UHF directly, or to be downlinked using the UHF transcevier)
- Decrypting incomming messages. (All uplinked messages will be encryped in accordance with CSA requirements)
- Bulk message handling
  - Fragmenting large outgoing messages into chunks that fit within the message unit size (paylod of Ax.25 packet) defined by the UHF transceiver. ***As of now this is 128 bytes***
  - Rebuilding a large incomming message from its constiuent chunks (which as of now will be 128 byte chunks)

## Usage

## Notes
