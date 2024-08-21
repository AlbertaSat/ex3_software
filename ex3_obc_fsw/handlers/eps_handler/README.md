# EPS HANDLER

This is the handler for the NanoAvionics EPS.

It its responsible for faciliating all interactions between the OBC FSW and the EPS peripheral hardware.

## Usage

To use this handler .....

## Testing & Development

Insides the [scripts](../../../scripts/) directory there are scripts to test the handler during development without the peripheral (eps subsystem) OR the message dispatcher.

This can be tested independent of the whole 'software stack' (coms handler, ground station, etc..) through the use of the [ipc_server_dev_tool](../../dev_tools/ipc_server_dev_tool/), which directly injects commands into the handler replacing the command dispatcher.
