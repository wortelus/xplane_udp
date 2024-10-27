use std::io;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;
use log::{error, info};
use crate::beacon::Beacon;
use crate::consts;

const BEACON_BUFFER_SIZE: usize = 1024;
type BeaconBuffer = [u8; BEACON_BUFFER_SIZE];

pub struct Session {
    beacon: Beacon,
    // xp_multicast_beacon_addr: SocketAddr,
    // xp_sending_address: SocketAddr,
    // xp_receiving_address: SocketAddr,
    xp_multicast_beacon_socket: UdpSocket,
    // xp_sending_socket: UdpSocket,
    // xp_receiving_socket: UdpSocket,
}

impl Session {
    pub fn auto_discover_default(ip_addr: IpAddr) -> Result<Self, Error> {
        let xp_multicast_beacon_socket = Self::create_beacon_socket(consts::XP_MULTICAST_ADDR)?;
        // let xp_sending_addr = SocketAddr::new(
        //     ip_addr,
        //     consts::XP_DEFAULT_SENDING_PORT);
        // let xp_receiving_addr = SocketAddr::new(
        //     ip_addr,
        //     consts::XP_DEFAULT_RECEIVING_PORT);
        // let xp_sending_socket = UdpSocket::bind(xp_sending_addr)?;
        // let xp_receiving_socket = UdpSocket::bind(xp_receiving_addr)?;

        Ok(Session {
            beacon: Beacon::default(),
            xp_multicast_beacon_socket,
        })
    }

    pub fn auto_discover(xp_receiving_address: SocketAddr,
                         xp_sending_address: SocketAddr) -> Result<Self, Error> {
        let xp_multicast_beacon_socket = Self::create_beacon_socket(consts::XP_MULTICAST_ADDR)?;
        // let xp_sending_socket = UdpSocket::bind(xp_sending_address)?;
        // let xp_receiving_socket = UdpSocket::bind(xp_receiving_address)?;

        Ok(Session {
            beacon: Beacon::default(),
            xp_multicast_beacon_socket,
        })
    }

    pub fn intercept_beacon(&mut self) {
        let mut buf = [0; BEACON_BUFFER_SIZE];

        loop {
            match self.xp_multicast_beacon_socket.recv_from(&mut buf) {
                Ok((size, src_addr)) => {
                    match self.parse_beacon_message(buf) {
                        Ok(_) => {
                            info!("Intercepted beacon: {:?}", self.beacon);
                        }
                        Err(e) => {
                            error!("Error parsing beacon message: {}", e);
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available yet
                }
                Err(e) => {
                    // Handle other errors
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            }

            // Optional: Add a small delay to prevent 100% CPU usage
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn get_beacon(&self) -> &Beacon {
        &self.beacon
    }

    fn create_beacon_socket(beacon_addr: SocketAddr) -> io::Result<UdpSocket> {
        info!("Connecting to X-Plane multicast group: {}", beacon_addr);
        match beacon_addr.ip() {
            IpAddr::V4(beacon_ip) => {
                if !beacon_ip.is_multicast() {
                    return Err(Error::new(io::ErrorKind::InvalidInput,
                                          "Expected a multicast address"));
                }

                let beacon_socket = UdpSocket::bind("0.0.0.0:49707")?;
                beacon_socket.join_multicast_v4(
                    // IP address of X-Plane multicast group
                    &beacon_ip,
                    // Listen on all interfaces
                    &Ipv4Addr::UNSPECIFIED,
                )?;
                beacon_socket.set_nonblocking(true)?;
                Ok(beacon_socket)
            }
            IpAddr::V6(_) => {
                Err(Error::new(io::ErrorKind::InvalidInput,
                               "Expected an IPv4 multicast address"))
            }
        }
    }

    fn parse_beacon_message(&mut self, msg: [u8; BEACON_BUFFER_SIZE]) -> Result<(), Box<dyn std::error::Error>> {
        let beacon = Beacon::from_bytes(&msg)?;
        self.beacon = beacon;
        Ok(())
    }
}