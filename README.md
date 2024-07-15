# ex3_software

All software from flight to ground station and everything in between for the epic mission of Ex-Alta 3. Contained is the directories for our software all within a single [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).

We decided to consolidate all software into a single repo to make testing easier and the ability to have shared functionality between the ground station and the OBC when it comes to how interfaces and messages are handled.

## ex3_obc_fsw

This directory is in charge of all command and data handling that happens on board the spacecraft. It includes all handlers for the payloads and deployables as well as the internal architecture for data handling.

## ex3_ground_station

This section acts as a mirror to the command and data handling that happens onboard. It takes in data that is sent from the spacecraft and makes data into the proper format to be sent up to the spacecraft. The architecture for this part of the mission can be found [here](https://docs.google.com/document/d/16SF8vcxaJGGWbYRoj0i6DKa5mFLjRM5MzQlzSKbrGHI/edit)

## ex3_shared_libs

Contained here is the shared functionality mentioned above between the ground station and the OBC. Mainly, serializing and deserializing messages and the required interfaces that allow for data to be passed from one process to another.
