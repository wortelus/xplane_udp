use std::io;
use std::net::{SocketAddrV4, UdpSocket};
use log::{debug, error};

use crate::beacon::Beacon;

pub struct AutoDiscover {
    pub beacon: Beacon,
}

impl AutoDiscover {
    pub async fn auto_discover_default(timeout: u64) -> Result<Self, io::Error> {
        let xp_receiving_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to receiving socket: {}", e);
                e
            })?;
        debug!("Receiving socket bound to {}", xp_receiving_socket.local_addr()?);

        let xp_sending_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to sending socket: {}", e);
                e
            })?;
        debug!("Sending socket bound to {}", xp_sending_socket.local_addr()?);

        Ok(AutoDiscover {
            beacon: Beacon::new(timeout).await?,
        })
    }

    pub async fn auto_discover(beacon_addr: SocketAddrV4,
                               timeout: u64) -> Result<Self, io::Error> {
        let xp_receiving_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to receiving socket: {}", e);
                e
            })?;
        debug!("Receiving socket bound to {}", xp_receiving_socket.local_addr()?);

        let xp_sending_socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| {
                error!("Failed to bind to sending socket: {}", e);
                e
            })?;
        debug!("Sending socket bound to {}", xp_sending_socket.local_addr()?);

        Ok(AutoDiscover {
            beacon: Beacon::new_with_address(beacon_addr, timeout).await?,
        })
    }

    pub fn get_beacon(&self) -> &Beacon {
        &self.beacon
    }

    pub fn get_beacon_mut(&mut self) -> &mut Beacon {
        &mut self.beacon
    }
}