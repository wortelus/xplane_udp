use std::fmt::Display;
use std::io;
use crate::consts::BEACON_PREFIX;
use crate::utils::parse_message_data;

#[derive(Debug, Default)]
pub struct BeaconData {
    beacon_major_version: u8,
    beacon_minor_version: u8,
    application_host_id: i32,
    version_number: i32,
    role: u32,
    port: u16,
    computer_name: String,
}

impl Display for BeaconData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Beacon {{ beacon_major_version: {}, \
               beacon_minor_version: {}, \
               application_host_id: {}, \
               version_number: {}, \
               role: {}, \
               port: {}, \
               computer_name: {} }}",
               self.beacon_major_version,
               self.beacon_minor_version,
               self.application_host_id,
               self.version_number,
               self.role,
               self.port,
               self.computer_name)
    }
}

impl BeaconData {
    pub fn new(beacon_major_version: u8,
               beacon_minor_version: u8,
               application_host_id: i32,
               version_number: i32,
               role: u32,
               port: u16,
               computer_name: String) -> BeaconData {
        BeaconData {
            beacon_major_version,
            beacon_minor_version,
            application_host_id,
            version_number,
            role,
            port,
            computer_name,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<BeaconData, io::Error> {
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
            application_host_id: i32::from_le_bytes(parse_message_data(&bytes[7..11])?),
            version_number: i32::from_le_bytes(parse_message_data(&bytes[11..15])?),
            role: u32::from_le_bytes(parse_message_data(&bytes[15..19])?),
            port: u16::from_le_bytes(parse_message_data(&bytes[19..21])?),
            computer_name: String::from_utf8_lossy(&bytes[21..end]).to_string(),
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
}