use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const XP_MULTICAST_GRP: IpAddr = IpAddr::V4(Ipv4Addr::new(239, 255, 1, 1));
pub const XP_MULTICAST_PORT: u16 = 49707;
pub const XP_MULTICAST_ADDR: SocketAddr = SocketAddr::new(XP_MULTICAST_GRP, XP_MULTICAST_PORT);

pub const XP_DEFAULT_RECEIVING_PORT: u16 = 49000;
pub const XP_DEFAULT_SENDING_PORT: u16 = 49001;

pub const XP_BEACON_PREFIX: &[u8; 4] = b"BECN";