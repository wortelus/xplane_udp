use log::{debug, error, info};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::beacon_data::BeaconData;
use crate::consts::{
    STANDARD_BUFFER_SIZE, XP_MULTICAST_ADDR, XP_MULTICAST_GRP,
    XP_MULTICAST_PARSE_MAX_TRIES, XP_MULTICAST_TIMEOUT_MAX_TRIES,
};

pub struct Beacon {
    data: Option<BeaconData>,
    xp_multicast_address: SocketAddrV4,
    xp_multicast_beacon_socket: UdpSocket,
    timeout_duration: Duration,
}

impl Beacon {
    pub async fn new(timeout: u64) -> io::Result<Self> {
        let socket = Self::init_beacon(XP_MULTICAST_ADDR).await?;
        let xp_multicast_address = XP_MULTICAST_ADDR;

        let per_attempt_timeout = Duration::from_millis(timeout / (XP_MULTICAST_TIMEOUT_MAX_TRIES as u64 + 1));

        Ok(Beacon {
            data: None,
            xp_multicast_address,
            xp_multicast_beacon_socket: socket,
            timeout_duration: per_attempt_timeout,
        })
    }

    pub async fn new_with_address(beacon_address: SocketAddrV4, timeout: u64) -> io::Result<Self> {
        let socket = Self::init_beacon(beacon_address).await?;
        let per_attempt_timeout = Duration::from_millis(timeout / (XP_MULTICAST_TIMEOUT_MAX_TRIES as u64 + 1));

        Ok(Beacon {
            data: None,
            xp_multicast_address: beacon_address,
            xp_multicast_beacon_socket: socket,
            timeout_duration: per_attempt_timeout,
        })
    }

    async fn init_beacon(beacon_address: SocketAddrV4) -> io::Result<UdpSocket> {
        if !beacon_address.ip().is_multicast() {
            error!("Invalid multicast address: {}", beacon_address.ip());
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid multicast address"));
        }

        let beacon_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, beacon_address.port())).await?;

        Ok(beacon_socket)
    }

    async fn connect_beacon(&mut self) -> io::Result<()> {
        info!("Connecting to X-Plane multicast group: {}", self.xp_multicast_address);

        self.xp_multicast_beacon_socket.join_multicast_v4(
            // IP address of X-Plane multicast group
            *self.xp_multicast_address.ip(),
            // Listen on all interfaces
            Ipv4Addr::UNSPECIFIED,
        )?;

        Ok(())
    }

    pub async fn close_beacon(&self) -> io::Result<()> {
        self.xp_multicast_beacon_socket.leave_multicast_v4(
            XP_MULTICAST_GRP,
            Ipv4Addr::UNSPECIFIED,
        )?;

        debug!("Left X-Plane multicast group: {}", self.xp_multicast_address);
        Ok(())
    }

    pub async fn intercept_beacon(&mut self) -> Result<(), io::Error> {
        let mut buf = [0; STANDARD_BUFFER_SIZE];
        let mut parse_tries = 1;
        let mut timeout_tries = 1;

        // Join multicast
        self.connect_beacon().await?;

        loop {
            // Try receiving within a timeout
            let recv_result = timeout
                (
                    self.timeout_duration,
                    self.xp_multicast_beacon_socket.recv_from(&mut buf),
                ).await;

            match recv_result {
                Ok(Ok((size, src_addr))) => {
                    let message = {
                        let mut msg_buf = [0u8; STANDARD_BUFFER_SIZE];
                        msg_buf[..size].copy_from_slice(&buf[..size]);
                        msg_buf
                    };

                    match self.parse_beacon_message(message, src_addr) {
                        Ok(_) => {
                            match &self.data {
                                Some(data) => {
                                    info!(
                                        "Intercepted beacon: from {} at {} running X-Plane {}",
                                        data.get_computer_name(),
                                        src_addr,
                                        data.get_version_number_string()
                                    );
                                }
                                None => {
                                    error!("Failed to parse beacon message");
                                    return Err(io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        "failed to parse beacon message",
                                    ));
                                }
                            }
                            debug!("Beacon data: {:?}", self.data);
                            return Ok(());
                        }
                        Err(e) => {
                            if parse_tries <= XP_MULTICAST_PARSE_MAX_TRIES {
                                debug!(
                                    "Error parsing beacon message: {}, retrying {}/{}",
                                    e, parse_tries, XP_MULTICAST_PARSE_MAX_TRIES
                                );
                                parse_tries += 1;
                            } else {
                                error!(
                                    "Failed to parse beacon message after {} tries",
                                    XP_MULTICAST_PARSE_MAX_TRIES
                                );
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "failed to parse beacon message",
                                ));
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    // Non-timeout related error receiving data
                    error!("Error receiving beacon messages: {}", e);
                    return Err(e);
                }
                Err(_elapsed) => {
                    // The timeout future elapsed, meaning recv_from did not complete in time
                    if timeout_tries <= XP_MULTICAST_TIMEOUT_MAX_TRIES {
                        debug!(
                            "Timeout receiving beacon message, retrying {}/{}",
                            timeout_tries, XP_MULTICAST_TIMEOUT_MAX_TRIES
                        );
                        timeout_tries += 1;
                    } else {
                        error!(
                            "Failed to receive beacon message after {} tries",
                            XP_MULTICAST_TIMEOUT_MAX_TRIES
                        );
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "failed to intercept X-Plane-beacon beacon",
                        ));
                    }
                }
            }
        }
    }

    fn parse_beacon_message(&mut self, msg: [u8; STANDARD_BUFFER_SIZE], src_addr: SocketAddr) -> Result<(), io::Error> {
        let beacon = BeaconData::from_bytes(&msg, src_addr)?;
        self.data = Some(beacon);
        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<BeaconData> {
        &self.data
    }

    pub fn get_address(&self) -> SocketAddrV4 {
        self.xp_multicast_address
    }
}