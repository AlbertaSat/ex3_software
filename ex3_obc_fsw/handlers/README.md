# Handlers

## Architectural Description

Handlers encapsulate functionality related to a particualr external peripheral such as any subsystem or payload. Handlers exist in a one to one relationship with a subsystem or payload.

Handlers act as middlemen for communication with their respective subsystem or payload. All communication with the associated subsystem or payload passes through its handler, and then into its interface.

## Implementation / Design

Handlers exist as seperate processes and communicate with the rest of the OBC FSW via interprocess communication.

Handlers communicate with their associated subsystem/payload using an interface that provides whichever communication protocol used by the device, although initially and for development purposes they initially are setup using TCP to communicate with simulated subsystems / payloads.

...

## Usage

Usage is defined for each handler in its associated README
