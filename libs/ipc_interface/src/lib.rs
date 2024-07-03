use nix::libc;
use nix::sys::socket::{self, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::unistd::{read, write};
use std::ffi::CString;
use std::io;
use std::path::Path;
use std::process;
use std::io::Error as IoError;

pub const BUFFER_SIZE: usize = 1024;
const CLIENT_POLL_TIMEOUT_MS: i32 = 100;

pub struct IPCInterface {
    fd: i32,
    socket_name: String,
    connected: bool,
}

impl IPCInterface {
    pub fn new(socket_name: String) -> IPCInterface {
        let mut ipc: IPCInterface = IPCInterface {
            fd: 0,
            socket_name: "string".to_string(),
            connected: false,
        };
        let fd = ipc.create_socket().unwrap();
        let connected = if ipc.make_connection(fd, socket_name.clone()) {
            true
        } else {
            false
        };
        IPCInterface {
            fd,
            socket_name,
            connected,
        }
    }

    /// create a socket of type SOCK_SEQPACKET to allow passing of information through processes
    fn create_socket(&mut self) -> io::Result<i32> {
        let socket_fd = socket::socket(
            AddressFamily::Unix,
            SockType::SeqPacket,
            SockFlag::empty(),
            None,
        )?;
        Ok(socket_fd)
    }

    /// Connect client process. True if connection is established.
    fn make_connection(&mut self, socket_fd: i32, client_name: String) -> bool {
        let fifo_name = format!("{}{}", SOCKET_PATH_PREPEND, client_name);
        let socket_path = CString::new(fifo_name).unwrap();
        let addr = UnixAddr::new(Path::new(socket_path.to_str().unwrap())).unwrap_or_else(|err| {
            eprintln!("Failed to create UnixAddr: {}", err);
            process::exit(1);
        });
        println!("Attempting to connect to {}", socket_path.to_str().unwrap());

        socket::connect(socket_fd, &addr).unwrap_or_else(|err| {
            eprintln!("Failed to connect to server: {}", err);
            process::exit(1);
        });

        println!(
            "Successfully Connected to {}, with fd: {}",
            socket_path.to_str().unwrap(),
            socket_fd
        );
        true
    }
}


/// read bytes over a UNIX SOCK_SEQPACKET socket from a sender. Takes in the fd location to write to.
/// loop{} over this
/// The user needs to create a buffer to pass to the read function.
pub fn read_socket(read_fd: i32, socket_buf: &mut Vec<u8>) -> Result<usize, IoError> {
    // client name is the name of the handler or thing that the client is trying to connect to (fifo is named with this in path)

    //We assume the fd for stdin is always zero. This is the default for UNIX systems and is unlikely to change.

    let mut poll_fds = [libc::pollfd {
        fd: read_fd,
        events: libc::POLLIN,
        revents: 0,
    }];

    let ready = unsafe {
        libc::poll(
            poll_fds.as_mut_ptr(),
            poll_fds.len() as libc::nfds_t,
            CLIENT_POLL_TIMEOUT_MS,
        )
    };

    if ready == -1 {
        eprintln!("poll error");
        process::exit(1);
    }

    for poll_fd in &poll_fds {
        // println!("poll_fd: {:?}", poll_fd);
        if poll_fd.revents != 0 {
            if poll_fd.revents & libc::POLLIN != 0 {
                if poll_fd.fd == read_fd {
                    let ret = read(read_fd, socket_buf).unwrap();

                    if ret == 0 {
                        println!("Connection to server dropped. Exiting...");
                        process::exit(0);
                    } else {
                        println!("Received: {:?}", socket_buf);
                    }
                    return Ok(ret);
                }
            }
        }
    }
    return Ok(0);
}

/// Function for sending data over a specific socket fd. The data should be a
/// serialized Msg struct as a Vec<u8>
pub fn send_over_socket(write_fd: i32, data: Vec<u8>) -> Result<usize, IoError> {
    Ok(write(write_fd, data.as_slice()).unwrap_or_else(|_| {
        eprintln!("write error");
        process::exit(1);
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dfgm_echo() {
        let interface = IPCInterface::new("dfgm_handler".to_string());
        let mut socket_buf = vec![0u8; IPC_BUFFER_SIZE];
        loop {
            let output = read_socket(interface.fd, &mut socket_buf).unwrap();
            if output > 5 {
                break;
            } else {
                continue;
            }
        }
        println!("Sending: {:?}", socket_buf);
        send_over_socket(interface.fd, socket_buf.clone()).unwrap();
    }
}