# Shell Handler

The Shell payload receives linux commands from the ground station and executes
them on the OBC. The output of the command is returned to the ground station as
the response.

### Example

<dl>
  <dt>Message</dt>
    <dd>`echo hello`</dd>
  <dt>Response</dt>
    <dd>`hello`</dd>
</dl>

The Cargo.toml will have to include the 'common' module defined in the ex3_shared_libs directory so it can refer to the component ids and opcodes used throughout the SC.
