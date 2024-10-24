extern crate serialport;
use serialport::prelude::*;
use std::io::{self, Write, Read};
use std::time::Duration;

/// Interface trait to be implemented by all external interfaces
pub trait Interface {
    /// Send byte data to the interface as a shared slice type byte. Return number of bytes sent
    fn send(&mut self, data: &[u8]) -> Result<usize, io::Error>;
    /// Read byte data from the interface into a byte slice buffer. Return number of bytes read
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error>;
}

/// UARTInterface struct that implements the Interface trait
pub struct UARTInterface {
    port: Box<dyn SerialPort>,
}

impl Interface for UARTInterface {
    fn send(&mut self, data: &[u8]) -> Result<usize, io::Error> {
        self.port.write(data)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, io::Error> {
        self.port.read(buffer)
    }
}

impl UARTInterface {
    pub fn new(port_name: &str, baud_rate: u32) -> io::Result<Self> {
        let settings = SerialPortSettings {
            baud_rate,
            ..Default::default()
        };
        let port = serialport::open_with_settings(port_name, &settings)?;
        Ok(UARTInterface { port })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Mock serial port for testing
    struct MockSerialPort {
        buffer: Cursor<Vec<u8>>,
    }

    /// Implement the SerialPort trait for MockSerialPort
    impl SerialPort for MockSerialPort {
        fn name(&self) -> Option<String> {
            Some("MockSerialPort".to_string())
        }

        fn settings(&self) -> SerialPortSettings {
            SerialPortSettings::default()
        }

        fn baud_rate(&self) -> serialport::Result<u32> {
            Ok(9600)
        }

        fn data_bits(&self) -> serialport::Result<DataBits> {
            Ok(DataBits::Eight)
        }

        fn flow_control(&self) -> serialport::Result<FlowControl> {
            Ok(FlowControl::None)
        }

        fn parity(&self) -> serialport::Result<Parity> {
            Ok(Parity::None)
        }

        fn stop_bits(&self) -> serialport::Result<StopBits> {
            Ok(StopBits::One)
        }

        fn timeout(&self) -> Duration {
            Duration::from_secs(1)
        }

        fn set_baud_rate(&mut self, _: u32) -> serialport::Result<()> {
            Ok(())
        }

        fn set_data_bits(&mut self, _: DataBits) -> serialport::Result<()> {
            Ok(())
        }

        fn set_flow_control(&mut self, _: FlowControl) -> serialport::Result<()> {
            Ok(())
        }

        fn set_parity(&mut self, _: Parity) -> serialport::Result<()> {
            Ok(())
        }

        fn set_stop_bits(&mut self, _: StopBits) -> serialport::Result<()> {
            Ok(())
        }

        fn set_timeout(&mut self, _: Duration) -> serialport::Result<()> {
            Ok(())
        }

        fn write(&mut self, data: &[u8]) -> serialport::Result<usize> {
            self.buffer.get_mut().extend_from_slice(data);
            Ok(data.len())
        }

        fn read(&mut self, data: &mut [u8]) -> serialport::Result<usize> {
            self.buffer.read(data).map_err(|e| serialport::Error::new(serialport::ErrorKind::Io, e))
        }
    }
    /// Test UARTInterface send function
    #[test]
    fn test_uart_interface_send() {
        let mut mock_port = MockSerialPort {
            buffer: Cursor::new(Vec::new()),
        };
        let mut interface = UARTInterface { port: Box::new(mock_port) };
        let data = b"Hello, UART!";
        let result = interface.send(data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data.len());
    }
    /// Test UARTInterface read function
    #[test]
    fn test_uart_interface_read() {
        let mut mock_port = MockSerialPort {
            buffer: Cursor::new(b"Hello, UART!".to_vec()),
        };
        let mut interface = UARTInterface { port: Box::new(mock_port) };
        let mut buffer = vec![0; 12];
        let result = interface.read(&mut buffer);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12);
        assert_eq!(&buffer, b"Hello, UART!");
    }
}