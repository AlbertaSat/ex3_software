# I2C Interface
This library allows userspace programs to communicate with devices via the I2C communication protocol. It includes an I2cDeviceInterface Structure that allows the user to read and write message structures as well as raw bytes over the I2C interface.

## Features
- Constructor to construct the I2C interface.
- read and send implementation for trait Interface allows user to read and write message structures using I2C interface.
- read and send function implemented allowing for communication using raw bytes.

## dependencies
i2cdev = "0.6.1"
