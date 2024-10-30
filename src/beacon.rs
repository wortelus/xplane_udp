use log::{debug, error, info};
use std::io;
use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;
use crate::beacon_data::BeaconData;
use crate::consts;
use crate::consts::{BEACON_BUFFER_SIZE, XP_MULTICAST_PARSE_MAX_TRIES};

pub struct Beacon {
    data: BeaconData,
    xp_multicast_address: SocketAddrV4,
    xp_multicast_beacon_socket: UdpSocket,
}

impl Beacon {
    pub fn new() -> io::Result<Self> {
        let socket = Self::init_beacon(consts::XP_MULTICAST_PORT)?;

        Ok(Beacon {
            data: BeaconData::default(),
            xp_multicast_address: consts::XP_MULTICAST_ADDR,
            xp_multicast_beacon_socket: socket,
        })
    }

    pub fn new_with_address(beacon_addr: SocketAddrV4) -> io::Result<Self> {
        let socket = Self::init_beacon(beacon_addr.port())?;

        Ok(Beacon {
            data: BeaconData::default(),
            xp_multicast_address: consts::XP_MULTICAST_ADDR,
            xp_multicast_beacon_socket: socket,
        })
    }

    pub fn close_beacon(&self) -> io::Result<()> {
        self.xp_multicast_beacon_socket.leave_multicast_v4(
            &consts::XP_MULTICAST_GRP,
            &Ipv4Addr::UNSPECIFIED,
        )
    }

    fn init_beacon(port: u16) -> io::Result<UdpSocket> {
        debug!("Init beacon socket on port {}", port);
        let beacon_socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        beacon_socket.set_nonblocking(true)?;
        Ok(beacon_socket)
    }

    pub fn connect_beacon(&mut self, beacon_addr: IpAddr) -> io::Result<()> {
        info!("Connecting to X-Plane multicast group: {}", beacon_addr);

        match beacon_addr {
            IpAddr::V4(beacon_ip) => {
                if !beacon_ip.is_multicast() {
                    return Err(Error::new(io::ErrorKind::InvalidInput,
                                          "Expected a multicast address"));
                }

                self.xp_multicast_beacon_socket.join_multicast_v4(
                    // IP address of X-Plane multicast group
                    &beacon_ip,
                    // Listen on all interfaces
                    &Ipv4Addr::UNSPECIFIED,
                )?;

                Ok(())
            }
            IpAddr::V6(_) => {
                Err(Error::new(io::ErrorKind::InvalidInput,
                               "Expected an IPv4 multicast address"))
            }
        }
    }

    pub fn intercept_beacon(&mut self) -> Result<(), Box<dyn error::Error>>{
        let mut buf = [0; BEACON_BUFFER_SIZE];
        let mut tries = 0;

        loop {
            match self.xp_multicast_beacon_socket.recv_from(&mut buf) {
                Ok((.., src_addr)) => {
                    match self.parse_beacon_message(buf) {
                        Ok(_) => {
                            info!("Intercepted beacon: from {} at {} running X-Plane {}",
                                self.data.get_computer_name(),
                                src_addr,
                                self.data.get_version_number_string());
                            return Ok(());
                        }
                        Err(e) => {
                            if tries <= XP_MULTICAST_PARSE_MAX_TRIES {
                                debug!("Error parsing beacon message: {}, retrying {}/{}",
                                e, tries, XP_MULTICAST_PARSE_MAX_TRIES);
                                tries += 1;
                            } else {
                                error!("Failed to parse beacon message after {} tries",
                                    XP_MULTICAST_PARSE_MAX_TRIES);
                                return Err(e);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available yet
                    continue;
                }
                Err(e) => {
                    error!("Error receiving beacon messages: {}", e);
                    return Err(Box::new(e));
                }
            }

            // Add a small delay to prevent 100% CPU usage
            // TODO: Replace with something else
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn parse_beacon_message(&mut self, msg: [u8; BEACON_BUFFER_SIZE]) -> Result<(), Box<dyn error::Error>> {
        let beacon = BeaconData::from_bytes(&msg)?;
        self.data = beacon;
        Ok(())
    }

    pub fn get_beacon(&self) -> &BeaconData { &self.data }
    pub fn get_address(&self) -> SocketAddrV4 { self.xp_multicast_address }

}