# UHF Handler

The UHF handler's purpose is to modify simulated operating parameters and get UHF data. Currently the UHF handler is not integrated with the rest of the OBC software.

## Usage

To use the UHF handler first start by running the "uplink_command_msg.sh" in the scripts directory.

Assuming the ex3_simulated_subsystems directory is in the same level as the ex3_software directory:

``` bash
cd scripts
./uplink_command_msg.sh ../../ex3_simulated_subsystems
```

Next, focus into the terminal labelled "SIM GS" and send a command to the UHF. Other UHF commands can be found [here](https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0).

This command sets the beacon value to "BEACON" by using opcode "4" and then providing space separated ascii encoded bytes (base 10):

``` text
UHF 4 66 69 65 67 79 78
```

You can observe if the beacon is was set correctly by checking the output in the "COMS_HANDLER" terminal or the "SIM_GS" terminal:

SIM_GS Terminal:

``` text
UHF 4 66 69 65 67 79 78
Built msg: Msg { header: MsgHeader { msg_type: 0, msg_id: 0, dest_id: 13, source_id: 7, op_code: 4, msg_len: 13 }, msg_body: [66, 69, 65, 67, 79, 78] }
Sent 13 bytes to Coms handler
Received ACK: Msg { header: MsgHeader { msg_type: 0, msg_id: 0, dest_id: 7, source_id: 8, op_code: 200, msg_len: 9 }, msg_body: [79, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }
Received Data: [66, 69, 65, 67, 79, 78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

COMS_HANDLER Terminal:

``` text
TRACE Received UHF IPC Msg bytes
TRACE Dserd msg body len 4089
TRACE Sent command successfully
TRACE Command response: 21 bytes 
TRACE Set UHF Beacon to: BEACON
```

If you recieved these messages it means that the command was successfully sent and the new beacon value can be observed from the SIM_GS terminal.

## UHF Parameters and data

As mentioned previously the UHF handler is not integrated with the rest of the ex3 OBC software. It can only be tested using the simulated UHF (Which is also not currently integrated with OBC software). In the future both the UHF ahndler and simulated UHF will be integrated with the OBC software.

For now the UHF handler modifies simulated parameters on the UHF. For now these are generalized parameters for the purpose of testing and are not representative of actual UHF parameters. The following are simulated parameters the simulated UHF currently has:

- __Beacon String__: This is the string that is sent down to the GS as a beacon. By default it is the string "beacon". This value can be changed by the UHF handler by using the SetBeacon opcode. You can get the UHF beacon string value by the UHF handler by using the GetBeacon opcode.
- __Mode__: This is the "Mode" of the UHF that refers to the frequency and baud rate of the UHF. As of now this parameter provides no real functionality and is a dummy value to be changed and read by the UHF handler. By default its value is 0, and can be any valid integer. Can be set using SetMode opcode, and get using GetMode opcode.
