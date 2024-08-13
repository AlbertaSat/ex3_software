# IRIS Handler

To run this handler properly run the uplink script located in ex3_software/scripts/

**Must have simulated IRIS running to process commands**

Contains one interface for communication with the simulated IRIS over TCP and a second interface for Unix domain sockets that are used for internal communication. The TCP interface is created on the port specified in common::ports for the simulated environment. 

The handler takes the opcode and arguements sent and translates them to the format the simulated IRIS subsystem expects. It then receives and parses the response from the IRIS subsystem, images are saved at the location specified by IRIS_DATA_PATH, other commands expect relatively minor responses and are printed directly to the terminal. 
There are currently 10 opcodes programmed, detailed in-depth within the simulated subsystems IRIS repository, currently located [here](https://github.com/AlbertaSat/ex3_simulated_subsystems/tree/main/IRIS). The main ones are **1** to turn the camera sensor on/off, **0** to capture an image and **2** to fetch images.

### Run and Testing
Currently there is not a defined way to run the program by itself. It requires at minimum the simulated Iris subsystem running as well as message dispatcher. However, it is easier to just run the uplink script and specify the IRIS subsystem at the ground station terminal.

1. Run the uplink script, details on how to run it are located in the [here](../../../README.md)
2. To send commands to the handler, locate the ground station terminal and type ```IRIS <opcode> <arg1> <arg2>...``` the number of arguements required depend on the command.

An example of the main 3 commands; there are currently 10 opcodes available, some with parameters such as opcode 1 and 2, others with no parameters such as opcode 0:
```bash
Ex3 CLI GS > IRIS 1 1 # Turns IRIS sensor on
Built msg: Msg { header: MsgHeader { msg_len: 6, msg_id: 0, dest_id: 4, source_id: 7, op_code: 1 }, msg_body: [1] }
Sent 6 bytes to Coms handler
Received ACK: Msg { header: MsgHeader { msg_len: 7, msg_id: 0, dest_id: 7, source_id: 8, op_code: 200 }, msg_body: [79, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }

Ex3 CLI GS > IRIS 0 # Take a picture
Built msg: Msg { header: MsgHeader { msg_len: 5, msg_id: 0, dest_id: 4, source_id: 7, op_code: 0 }, msg_body: [] }
Sent 5 bytes to Coms handler
Received ACK: Msg { header: MsgHeader { msg_len: 7, msg_id: 0, dest_id: 7, source_id: 8, op_code: 200 }, msg_body: [79, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }

Ex3 CLI GS > IRIS 2 1 # Fetch one image
Built msg: Msg { header: MsgHeader { msg_len: 6, msg_id: 0, dest_id: 4, source_id: 7, op_code: 2 }, msg_body: [1] }
Sent 6 bytes to Coms handler
Received ACK: Msg { header: MsgHeader { msg_len: 7, msg_id: 0, dest_id: 7, source_id: 8, op_code: 200 }, msg_body: [79, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] }

Ex3 CLI GS > 
```