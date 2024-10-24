# Shell Handler

The Shell payload receives linux commands from the ground station and executes
them on the OBC. The output of the command is returned to the ground station as
the response.

## Example

<dl>
  <dt>Message</dt>
    <dd>`echo hello`</dd>
  <dt>Response</dt>
    <dd>`hello`</dd>
</dl>

## Example command

For testing if a command works, go to the SIM_GS and write:

```SHELL 0 echo hello```

The output for this on the GS should receive a message that equals 'hello'

For command outputs that take up more than 128B, multiple messages will be delivered to GS in response, each printed out.

The Cargo.toml will have to include the 'common' module defined in the ex3_shared_libs directory so it can refer to the component ids and opcodes used throughout the SC.
