use log::{error, info, debug};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::time::Duration;

use crate::consts::{XP_MULTICAST_ADDR, XP_MULTICAST_GRP,
                    STANDARD_BUFFER_SIZE, XP_MULTICAST_PARSE_MAX_TRIES,
                    XP_MULTICAST_TIMEOUT_MAX_TRIES};
use crate::beacon_data::BeaconData;

pub struct Beacon {
    data: Option<BeaconData>,
    xp_multicast_address: SocketAddrV4,
    xp_multicast_beacon_socket: UdpSocket,
}

impl Beacon {
    pub fn new(timeout: u64) -> io::Result<Self> {
        let socket = Self::init_beacon(XP_MULTICAST_ADDR)?;

        let mut beacon = Beacon {
            data: None,
            xp_multicast_address: XP_MULTICAST_ADDR,
            xp_multicast_beacon_socket: socket,
        };

        beacon.set_timeout(timeout)?;
        Ok(beacon)
    }

    pub fn new_with_address(beacon_address: SocketAddrV4, timeout: u64) -> io::Result<Self> {
        let socket = Self::init_beacon(beacon_address)?;

        let mut beacon = Beacon {
            data: None,
            xp_multicast_address: beacon_address,
            xp_multicast_beacon_socket: socket,
        };

        beacon.set_timeout(timeout)?;
        Ok(beacon)
    }

    fn init_beacon(beacon_address: SocketAddrV4) -> io::Result<UdpSocket> {
        let port = beacon_address.port();
        debug!("Init beacon socket on port {}", port);
        let beacon_socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        beacon_socket.set_nonblocking(false)?;

        if !beacon_address.ip().is_multicast() {
            error!("Invalid multicast address: {}", beacon_address.ip());
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid multicast address"));
        }

        Ok(beacon_socket)
    }

    pub fn close_beacon(&self) -> io::Result<()> {
        self.xp_multicast_beacon_socket.leave_multicast_v4(
            &XP_MULTICAST_GRP,
            &Ipv4Addr::UNSPECIFIED,
        )
    }

    pub fn connect_beacon(&mut self) -> io::Result<()> {
        info!("Connecting to X-Plane multicast group: {}", self.xp_multicast_address);

        self.xp_multicast_beacon_socket.join_multicast_v4(
            // IP address of X-Plane multicast group
            self.xp_multicast_address.ip(),
            // Listen on all interfaces
            &Ipv4Addr::UNSPECIFIED,
        )?;

        Ok(())
    }

    pub fn intercept_beacon(&mut self) -> Result<(), io::Error> {
        let mut buf = [0; STANDARD_BUFFER_SIZE];
        let mut parse_tries = 1;
        let mut timeout_tries = 1;

        loop {
            match self.xp_multicast_beacon_socket.recv_from(&mut buf) {
                Ok((.., src_addr)) => {
                    match self.parse_beacon_message(buf, src_addr) {
                        Ok(_) => {
                            match &self.data {
                                Some(data) => {
                                    info!("Intercepted beacon: from {} at {} running X-Plane {}",
                                        data.get_computer_name(),
                                        src_addr,
                                        data.get_version_number_string());
                                }
                                None => {
                                    error!("Failed to parse beacon message");
                                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                              "failed to parse beacon message",
                                    ));
                                }
                            }
                            debug!("Beacon data: {:?}", self.data);
                            return Ok(());
                        }
                        Err(e) => {
                            if parse_tries <= XP_MULTICAST_PARSE_MAX_TRIES {
                                debug!("Error parsing beacon message: {}, retrying {}/{}",
                                e, parse_tries, XP_MULTICAST_PARSE_MAX_TRIES);
                                parse_tries += 1;
                            } else {
                                error!("Failed to parse beacon message after {} tries",
                                    XP_MULTICAST_PARSE_MAX_TRIES);
                                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                          "failed to parse beacon message",
                                ));
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut || e.kind() == io::ErrorKind::WouldBlock => {
                    if timeout_tries <= XP_MULTICAST_TIMEOUT_MAX_TRIES {
                        debug!("Timeout receiving beacon message, retrying {}/{}",
                                timeout_tries, XP_MULTICAST_TIMEOUT_MAX_TRIES);
                        timeout_tries += 1;
                    } else {
                        error!("Failed to receive beacon message after {} tries",
                                    XP_MULTICAST_TIMEOUT_MAX_TRIES);
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                                                  "failed to intercept X-Plane-beacon beacon",
                        ));
                    }
                }
                Err(e) => {
                    error!("Error receiving beacon messages: {}", e);
                    return Err(e);
                }
            }
        }
    }

    pub fn set_timeout(&mut self, timeout: u64) -> io::Result<()> {
        debug!("Setting beacon socket timeout to {} ms", timeout);
        let timeout = timeout / (XP_MULTICAST_TIMEOUT_MAX_TRIES as u64 + 1);
        self.xp_multicast_beacon_socket.set_read_timeout(Some(Duration::from_millis(timeout)))
    }

    fn parse_beacon_message(&mut self, msg: [u8; STANDARD_BUFFER_SIZE], src_addr: SocketAddr) -> Result<(), io::Error> {
        let beacon = BeaconData::from_bytes(&msg, src_addr)?;
        self.data = Some(beacon);
        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<BeaconData> { &self.data }
    pub fn get_address(&self) -> SocketAddrV4 { self.xp_multicast_address }
}