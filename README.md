# ex3_software

All software from flight to ground station and everything in between for the epic mission of Ex-Alta 3. Contained is the directories for our software all within a single [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

We decided to consolidate all software into a single repo to make testing easier and the ability to have shared functionality between the ground station and the OBC when it comes to how interfaces and messages are handled.

## ex3_obc_fsw

This directory is in charge of all command and data handling that happens on board the spacecraft. It includes all handlers for the payloads and deployables as well as the internal architecture for data handling.

## ex3_ground_station

This section acts as a mirror to the command and data handling that happens onboard. It takes in data that is sent from the spacecraft and makes data into the proper format to be sent up to the spacecraft. The architecture for this part of the mission can be found [here](https://docs.google.com/document/d/16SF8vcxaJGGWbYRoj0i6DKa5mFLjRM5MzQlzSKbrGHI/edit)

## ex3_shared_libs

Contained here is the shared functionality mentioned above between the ground station and the OBC. Mainly, serializing and deserializing messages and the required interfaces that allow for data to be passed from one process to another.

## General Usage / Scripts

Scripts to run various sections of the software together can be found in the [scripts](./scripts) directory.

(For now) These scripts use bash and gnome-terminal which is are the standard shell and terminal shipped with Ubuntu systems. *Additionally*, these scripts also use the ex3_simulated_subsystem repo which is on the main AlbertaSat Github page. One must clone this, then provide the relative path to it when running the script as the first command line argument.

### Testing Uplink

Commands for uplink can be found in a master spreadsheet [here].(https://docs.google.com/spreadsheets/d/1rWde3jjrgyzO2fsg2rrVAKxkPa2hy-DDaqlfQTDaNxg/edit?gid=0#gid=0)

One can send a command by running the uplink script: ```uplink_command_msg.sh```.

```@sh
./uplink_command_msg <relative_path_to_sim_subsystem_dir>
```

Next, an operator will send commands from the Ground Station. Right now, it is the SIM_GS terminal that is spawned by the script. Next, type in a command structured as ```<DEST> <opcode> <body>(optional)```.

An example to toggle the collection of DFGM data would be:

```@sh
DFGM 0 1
```

followed by:

```@sh
DFGM 0 1
```

to toggle off the data collection. **NOTE**: this method of data collection is unique to the DFGM subsystem. Other subsystems will be able to collect data without having to toggle a flag.

### Testing Downlink  

Once all the processes are running, Send the command:

```@sh
BulkMsgDispatcher <onboard_path>
```

in the CLI_GS. The GS expects any path that is onboard to the data that it will slice and downlink. This will commence the bulk data transfer from the payload handler to the GS. One can run a diff on the created file from the GS and the data in the  *dfgm_data* folder to ensure everything was copied down correctly.

Since this path depends on where the bulk_msg_dispatcher is in the OBC flight software, an example of this command could look like:

```@sh
BulkMsgDispatcher ../handlers/dfgm_handler/dfgm_data
```
