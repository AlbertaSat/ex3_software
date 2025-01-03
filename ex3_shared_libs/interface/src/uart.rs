use nix::{unistd, fcntl, libc};
use nix::sys::termios::{self, ControlFlags, InputFlags, LocalFlags, OutputFlags, Termios, SpecialCharacterIndices};
use nix::sys::stat::Mode;
use std::os::fd::{FromRawFd, OwnedFd, AsRawFd, RawFd};
use std::fs::File;
use super::Interface;

mod raw {
    use nix::{libc, ioctl_read_bad};
    // define ioctl function for reading input buffer
    ioctl_read_bad!(fionread, libc::FIONREAD, libc::c_int);
}

pub fn fionread(fd: RawFd) -> Result<u32, std::io::Error> {
    let mut retval: libc::c_int = 0;
    unsafe { raw::fionread(fd, &mut retval) }
        .map(|_| retval as u32)
        .map_err(|e| e.into())
}

/// Sets the parity of the serial port.
/// The default value in serial port settings struct is None.
pub enum Parity {
    None = 0,
    Odd = 1,
    Even = 2,
}

/// Sets the stop bits of the serial port.
/// The default value in serial port settings struct is One.
pub enum StopBits {
    One = 0,
    Two = 1,
}

/// Sets the number of data bits parameter for the serial port.
/// The default value in serial port settings struct is eight.
pub enum DataBits {
    Five = 0,
    Six = 1,
    Seven = 2,
    Eight = 3,
}

pub struct SerialPortSettings {
    pub baud_rate: termios::BaudRate,
    pub data_bits: DataBits,
    pub parity: Parity,
    pub stop_bits: StopBits,
}

impl SerialPortSettings {
    /// Function to create new serial port settings struct. The port is created with the following
    /// default settings:
    /// Baud rate: 9600 bps,
    /// Data bits: 8,
    /// No parity bits,
    /// One stop bit,
    /// No flow control
    /// After creation of serial port settings you can still modify the settings before creating
    /// your serial port instance using UartInterface struct. These settings are only applied
    /// during the construction of UartInterface.
    pub fn new() -> Self {
        // Initalize all enums for serial port settings to default values
        SerialPortSettings {
            baud_rate: termios::BaudRate::B9600,
            data_bits: DataBits::Eight,
            parity: Parity::None,
            stop_bits: StopBits::One,
        }
    }
}

impl Default for SerialPortSettings {
    fn default() -> Self {
        SerialPortSettings::new()
    }
}

pub struct UartInterface {
    pub fd: OwnedFd,
    file_path: String,
}

impl Interface for UartInterface {
    fn send(&mut self, buff: &[u8]) -> Result<usize, std::io::Error> {
        let bytes_written = unistd::write(&self.fd, buff)?;
        Ok(bytes_written)
    }

    fn read(&mut self, buff: &mut [u8]) -> Result<usize, std::io::Error> {
        let bytes_read = unistd::read(self.fd.as_raw_fd(), buff)?;
        Ok(bytes_read)
    }
}

impl UartInterface {
    pub fn new(file_path: &str, settings_option: Option<&SerialPortSettings>) -> Self {
        let serial_port_fd_raw = fcntl::open(file_path, fcntl::OFlag::O_RDWR, Mode::S_IWUSR)
                                .expect("Could not open serial port.");
        let serial_port_fd = unsafe { File::from_raw_fd(serial_port_fd_raw) };
        let mut tty = termios::tcgetattr(&serial_port_fd).expect("Error getting serial port settings");

        // Create new instance of default settings if user did not provide any
        let settings = match settings_option {
            Some(s) => s,
            None => {
                &SerialPortSettings::new()
            }
        };

        // Applys settings in given by the serial port settings struct
        apply_settings(&mut tty, settings);

        // Setting VTIME to 0 and VMIN to 0 means the read returns instantly
        tty.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
        tty.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;

        // Set baud rate
        termios::cfsetspeed(&mut tty, settings.baud_rate).unwrap();
        // Apply settings immediately
        termios::tcsetattr(&serial_port_fd, termios::SetArg::TCSANOW, &tty).unwrap();

        UartInterface {fd: serial_port_fd.into(), file_path: file_path.to_string()}
    }

    // Flushes bytes in output bufffer that have not been transmitted yet
    pub fn flush_output(&mut self, ) -> Result<(), std::io::Error> {
        termios::tcflush(&self.fd, termios::FlushArg::TCOFLUSH)?;
        Ok(())
    }
    // Flushes bytes in input buffer that have not been read yet
    pub fn flush_input(&mut self, ) -> Result<(), std::io::Error> {
        termios::tcflush(&self.fd, termios::FlushArg::TCIFLUSH)?;
        Ok(())
    }

    pub fn get_file_path(&mut self) -> String {
        self.file_path.clone()
    }

    // Returns the bytes available to read in the serial buffer
    pub fn available_to_read(&self) -> Result<u32, std::io::Error> {
        fionread(self.fd.as_raw_fd())
    }
}


// helper function to make UART interface constructor more readable
fn apply_settings(tty: &mut Termios, settings: &SerialPortSettings) {
    // Disable all input processing flags
    tty.input_flags &= !(
        InputFlags::IXON |
        InputFlags::IXOFF |
        InputFlags::IXANY |
        InputFlags::IGNBRK |
        InputFlags::BRKINT |
        InputFlags::INLCR |
        InputFlags::IGNCR |
        InputFlags::PARMRK |
        InputFlags::ISTRIP
    );

    // Disable output processing flags
    tty.output_flags &= !( OutputFlags::OPOST | OutputFlags::ONLCR);

    tty.local_flags &= !(
        LocalFlags::ICANON |
        LocalFlags::ECHO |
        LocalFlags::ECHOE |
        LocalFlags::ECHONL |
        LocalFlags::ISIG
    );

    // Clear data size bits
    tty.control_flags &= !(ControlFlags::CSIZE | ControlFlags::CRTSCTS);

    // Enable reading from serial port and ignore modem control lines.
    tty.control_flags |= ControlFlags::CREAD | ControlFlags::CLOCAL;

    match settings.data_bits {
        DataBits::Five => {
            tty.control_flags |= ControlFlags::CS5
        }
        DataBits::Six => {
            tty.control_flags |= ControlFlags::CS6
        }
        DataBits::Seven => {
            tty.control_flags |= ControlFlags::CS7
        }
        DataBits::Eight => {
            tty.control_flags |= ControlFlags::CS8
        }
    }

    match settings.parity {
        Parity::None => {
            tty.control_flags &= !(ControlFlags::PARENB);
        },
        Parity::Odd => {
            tty.control_flags |= ControlFlags::PARENB;
            tty.control_flags |= ControlFlags::PARODD;
        },
        Parity::Even => {
            tty.control_flags |= ControlFlags::PARENB;
            tty.control_flags &= !(ControlFlags::PARODD);
        }
    }

    match settings.stop_bits {
        StopBits::One => {
            tty.control_flags &= !(ControlFlags::CSTOPB);
        }
        StopBits::Two => {
            tty.control_flags |= ControlFlags::CSTOPB;
        }
    }
}


