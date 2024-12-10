# Shared libraries for Ex-Alta3 code

common: library that is shared between the Ground Station (GS) and the On Board
    Computer (OBC) Flight Software (FSW) for the Ex-Alta 3 mission. Some of
    the shared modules are:
    - ports: ports used by the payloads for inter-component communication
    - component_ids: definitions of payload IDs
    - message_structure: bulk/cmd/response message formats
    - logging: time-stamped logging facility

interface: library of I/O interface helpers that is shared among the handlers
   on the OBC. Some of the modules are:
   - ipc: inter-payload communication client/server support
   - i2c: I2C communication support
   - uart: serial port support
   - tcp: client and server tcp support
