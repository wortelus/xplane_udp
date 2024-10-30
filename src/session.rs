use std::io;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use log::{debug, error};

use crate::beacon::Beacon;

pub struct Session {
    beacon: Option<Beacon>,
    xp_receiving_socket: UdpSocket,
}

impl Session {
    pub fn auto_discover_default() -> Result<Self, io::Error> {
        let xp_receiving_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to receiving socket: {}", e);
                e
            })?;

        Ok(Session {
            beacon: Some(Beacon::new()?),
            xp_receiving_socket,
        })
    }

    pub fn auto_discover(beacon_addr: SocketAddrV4,
                         xp_receiving_address: SocketAddr) -> Result<Self, io::Error> {
        let xp_receiving_socket = UdpSocket::bind(xp_receiving_address)
            .map_err(|e| {
                error!("Failed to bind to receiving socket on {}: {}", xp_receiving_address, e);
                e
            })?;

        Ok(Session {
            beacon: Some(Beacon::new_with_address(beacon_addr)?),
            xp_receiving_socket,
        })
    }

    pub fn connect(&mut self) -> Result<(), io::Error> {
        match self.beacon {
            Some(ref mut beacon) => {
                beacon.connect_beacon()?;
            }
            None => {
                debug!("Beacon not initialized, skipping multicast connection");
            }
        };
        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<Beacon> {
        &self.beacon
    }
}