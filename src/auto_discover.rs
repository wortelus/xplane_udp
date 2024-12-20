use std::io;
use std::net::SocketAddrV4;

use crate::beacon::Beacon;

pub struct AutoDiscover {
    pub beacon: Beacon,
}

impl AutoDiscover {
    pub async fn auto_discover_default(timeout: u64) -> Result<Self, io::Error> {
        Ok(AutoDiscover {
            beacon: Beacon::new(timeout).await?,
        })
    }

    pub async fn auto_discover(beacon_addr: SocketAddrV4,
                               timeout: u64) -> Result<Self, io::Error> {
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