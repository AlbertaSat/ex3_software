# ex3_ground_station

Ground station software to interface with Ex Alta 3

This (very) preliminary strawman uses Rocket to implement a simple WebServer
that can send messages to the OBC prototype in ex3_obc_fe2o3.

To build/run:
```bash
$ cargo run
```

This should launch the http-server on http://localhost:8000/index.html
The server currently expects the OBC to be listening on localhost:50000

