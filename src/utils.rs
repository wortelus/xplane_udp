use std::io;

pub fn parse_message_data<const N: usize>(bytes: &[u8]) -> Result<[u8; N], io::Error> {
    bytes.try_into().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid slice length"))
}