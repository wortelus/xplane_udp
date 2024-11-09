use std::io;
use std::net::SocketAddr;

use crate::consts::BEACON_PREFIX;

#[derive(Debug)]
pub struct BeaconData {
    /// Source address of the beacon message
    source: SocketAddr,

    beacon_major_version: u8,
    beacon_minor_version: u8,
    application_host_id: i32,
    version_number: i32,
    role: u32,
    port: u16,
    computer_name: String,
}

impl BeaconData {
    pub fn new(beacon_major_version: u8,
               beacon_minor_version: u8,
               application_host_id: i32,
               version_number: i32,
               role: u32,
               port: u16,
               computer_name: String,
               source: SocketAddr) -> BeaconData {
        BeaconData {
            beacon_major_version,
            beacon_minor_version,
            application_host_id,
            version_number,
            role,
            port,
            computer_name,
            source
        }
    }

    pub fn from_bytes(bytes: &[u8], src_addr: SocketAddr) -> Result<BeaconData, io::Error> {
        if bytes.len() < 21 {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "beacon message too short"),
            );
        }

        if !bytes.starts_with(BEACON_PREFIX) {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "not a beacon message")
            );
        }

        // First null byte indicates end of computer name
        let end = 21 + bytes[21..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(bytes.len());

        Ok(BeaconData {
            beacon_major_version: bytes[5],
            beacon_minor_version: bytes[6],
            application_host_id: i32::from_le_bytes([bytes[7], bytes[8], bytes[9], bytes[10]]),
            version_number: i32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]),
            role: u32::from_le_bytes([bytes[15], bytes[16], bytes[17], bytes[18]]),
            port: u16::from_le_bytes([bytes[19], bytes[20]]),
            computer_name: String::from_utf8_lossy(&bytes[21..end]).to_string(),
            source: src_addr,
        })
    }

    pub fn get_major_version(&self) -> u8 { self.beacon_major_version }
    pub fn get_minor_version(&self) -> u8 { self.beacon_minor_version }
    pub fn get_application_host_id(&self) -> i32 { self.application_host_id }
    pub fn get_version_number(&self) -> i32 { self.version_number }
    pub fn get_version_number_string(&self) -> String {
        format!("{}.{}.{}", self.version_number / 10000, self.version_number / 100 % 100, self.version_number % 100)
    }
    pub fn get_role(&self) -> u32 { self.role }
    pub fn get_port(&self) -> u16 { self.port }
    pub fn get_computer_name(&self) -> &str { &self.computer_name }
    pub fn get_source(&self) -> &SocketAddr { &self.source }
}