# CAN Interface

## Testing with a Virtual CAN Interface

Make sure the `vcan` module is loaded

```sh
sudo modprobe vcan
```

Use the `ip` command to create a `vcan` interface named `vcan0`

```sh
sudo ip link add dev vcan0 type vcan
```

Then activate `vcan0` with

```sh
sudo ip link set up vcan0
```

The `vcan` is ready for use and testing.