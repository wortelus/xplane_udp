use std::io;
use std::net::{SocketAddr, UdpSocket};
use log::{debug};

pub struct CommandHandler {}

impl CommandHandler {
    pub fn new() -> CommandHandler {
        CommandHandler {}
    }

    fn cmd_message(&self, command: &str) -> String {
        format!("CMND\0{}\0", command)
    }

    pub fn send_command(&self, command: &str, sending_socket: &UdpSocket, receiving_address: &SocketAddr) -> io::Result<()> {
        debug!("Sending command {}", command);
        let message = self.cmd_message(command);
        sending_socket.send_to(message.as_bytes(), receiving_address)?;
        Ok(())
    }
}