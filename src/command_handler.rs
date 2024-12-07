use std::io;
use std::net::{SocketAddr, UdpSocket};
use log::{debug};

use crate::consts::ALRT_PREFIX;

// TODO: better alert system
#[derive(Debug, Default)]
pub struct AlertMessage {
    lines: [String; 4],
}

impl AlertMessage {
    pub fn set_line(&mut self, line: &str, index: usize) -> io::Result<()> {
        if index > 3 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      "You have lines 0-3 available"));
        }
        if line.len() > 240 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      "Line 1 is too long"));
        }
        self.lines[index] = line.to_string();
        Ok(())
    }
}

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

    fn alert_message(&self, alert_message: AlertMessage) -> Vec<u8> {
        // <4sx240s240s240s240s
        // <: little-endian
        // 4s: 4 byte string
        // x: pad byte
        // 240s: 240 byte string (4 lines of 60 characters)
        // ref: https://xppython3.readthedocs.io/en/latest/development/udp/alrt.html

        let len: usize = 5 + 4 * 240;
        let mut message: Vec<u8> = vec![0; len];

        let len_1 = alert_message.lines[0].len();
        let len_2 = alert_message.lines[1].len();
        let len_3 = alert_message.lines[2].len();
        let len_4 = alert_message.lines[3].len();

        message[0..4].copy_from_slice(ALRT_PREFIX);
        message[4] = 0;
        message[5..5+len_1].copy_from_slice(alert_message.lines[0].as_bytes());
        message[245..245+len_2].copy_from_slice(alert_message.lines[1].as_bytes());
        message[485..485+len_3].copy_from_slice(alert_message.lines[2].as_bytes());
        message[725..725+len_4].copy_from_slice(alert_message.lines[3].as_bytes());

        message
    }

    pub fn alert(&self, alert_message: AlertMessage, sending_socket: &UdpSocket, receiving_address: &SocketAddr) -> io::Result<()> {
        debug!("Sending alert");
        let message = self.alert_message(alert_message);
        sending_socket.send_to(message.as_slice(), receiving_address)?;
        Ok(())
    }
}