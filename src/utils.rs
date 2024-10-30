use std::io;

pub fn slice_to_array<const N: usize>(bytes: &[u8]) -> Result<[u8; N], io::Error> {
    bytes.try_into().map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid slice length"))
}