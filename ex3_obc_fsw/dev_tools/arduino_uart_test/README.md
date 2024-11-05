# Arduino UART Test
This is an extremely simple test program for the purposes of testing reading capabilities of the UART interface using the Arduino and serial communication.

## How to Use
Start by connecting an Arduino Uno to your computer via the USB cable provided with the board. Next, open the Arduino IDE, connect your board to the serial port "/dev/ttyACM0". 

Next upload the following simple sketch to continuously send the ASCII character 1 to the test program:

``` C
void setup() {
  Serial.begin(9600);
}

void loop() {
  Serial.write("1");
}

```

Once the program is successfully uploaded to your Arduino start the test program using the following command:

``` bash
cargo run
```

If successful you will see a continuous stream of "49" (ASCII 1 character) filling the buffer.
