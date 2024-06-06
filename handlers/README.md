# Handlers

Handlers encapsulate functionality related to a particualr external peripheral such as any subsystem or payload.

Handlers exist as seperate processes and communicate with the rest of the OBC FSW via interprocess communication.

.Handlers host interfaces for communication with their respective subsystem or payload. All communication with the associated subsystem or payload passes through its handler, and then into its interface.
