use std::net::{Ipv4Addr, SocketAddrV4};

// ─── Multicast communication default values ────────────────────────────────────────
/// Default multicast group for X-Plane
pub const XP_MULTICAST_GRP: Ipv4Addr = Ipv4Addr::new(239, 255, 1, 1);

/// Default multicast port for X-Plane
pub const XP_MULTICAST_PORT: u16 = 49707;

/// Default multicast address for X-Plane
pub const XP_MULTICAST_ADDR: SocketAddrV4 = SocketAddrV4::new(XP_MULTICAST_GRP, XP_MULTICAST_PORT);

/// Default multicast timeout for X-Plane
pub const XP_MULTICAST_TIMEOUT_MAX_TRIES: i32 = 3;

/// Maximum number of tries to parse a multicast message
pub const XP_MULTICAST_PARSE_MAX_TRIES: i32 = 3;


// ─── IP UDP Communication ports ──────────────────────────────────────────────────────
pub const XP_DEFAULT_RECEIVING_PORT: u16 = 49000;
pub const XP_DEFAULT_SENDING_PORT: u16 = 49001;

// ─── Message constants ───────────────────────────────────────────────────────
pub const STANDARD_BUFFER_SIZE: usize = 1024;
pub const BEACON_PREFIX: &[u8; 4] = b"BECN";
pub const RREF_PREFIX: &[u8; 4] = b"RREF";
