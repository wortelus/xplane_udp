use std::io;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use log::{debug, error, info};

use crate::beacon::Beacon;
use crate::consts::XP_DEFAULT_RECEIVING_PORT;

pub struct Session {
    beacon: Option<Beacon>,
    xp_receiving_address: Option<SocketAddr>,
    xp_receiving_socket: UdpSocket,
}

impl Session {
    pub fn auto_discover_default(timeout: u64) -> Result<Self, io::Error> {
        let xp_receiving_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to receiving socket: {}", e);
                e
            })?;

        Ok(Session {
            beacon: Some(Beacon::new(timeout)?),
            xp_receiving_address: None,
            xp_receiving_socket,
        })
    }

    pub fn auto_discover(beacon_addr: SocketAddrV4,
                         xp_receiving_address: SocketAddr,
                         timeout: u64) -> Result<Self, io::Error> {
        Ok(Session {
            beacon: Some(Beacon::new_with_address(beacon_addr, timeout)?),
            xp_receiving_address: Some(xp_receiving_address),
            xp_receiving_socket: UdpSocket::bind("0.0.0.0:0")
                .map_err(|e| {
                    error!("Failed to bind to receiving socket: {}", e);
                    e
                })?,
        })
    }

    pub fn manual(xp_receiving_address: SocketAddr) -> Result<Self, io::Error> {
        Ok(Session {
            beacon: None,
            xp_receiving_address: Some(xp_receiving_address),
            xp_receiving_socket: UdpSocket::bind(xp_receiving_address)
                .map_err(|e| {
                    error!("Failed to bind to receiving socket: {}", e);
                    e
                })?,
        })
    }

    pub async fn connect(&mut self) -> Result<(), io::Error> {
        match self.beacon {
            Some(ref mut beacon) => {
                beacon.connect_beacon()?;
                beacon.intercept_beacon().await?;
                if let Some(ref addr) = self.xp_receiving_address {
                    debug!("Connecting to X-Plane at {}", addr);
                    self.xp_receiving_socket.connect(addr)?;
                } else {
                    debug!("No X-Plane address provided, auto-discovering from beacon...");
                    let beacon_data = match beacon.get_beacon() {
                        Some(data) => data,
                        None => {
                            error!("No beacon data available, cannot auto-discover X-Plane");
                            return Err(io::Error::new(io::ErrorKind::NotFound, "No beacon data available"));
                        }
                    };
                    let addr = SocketAddr::new(
                        beacon_data.get_source().ip(),
                        beacon_data.get_port(),
                    );
                    self.connect_xp(addr)?;
                }
            }
            None => {
                debug!("Beacon not initialized, skipping multicast connection");
            }
        };
        Ok(())
    }

    fn connect_xp(&mut self, addr: SocketAddr) -> Result<(), io::Error> {
        info!("Connecting to X-Plane at {}", addr);
        self.xp_receiving_address = Some(addr);
        self.xp_receiving_socket.connect(addr)?;
        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<Beacon> {
        &self.beacon
    }
}