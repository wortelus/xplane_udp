use std::io;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::Arc;
use log::{debug, error, info};

use crate::consts::XP_DEFAULT_SENDING_PORT;
use crate::beacon::Beacon;
use crate::command_handler::{AlertMessage, CommandHandler};
use crate::dataref_type::{DataRefType, DataRefValueType};
use crate::dataref_handler::DataRefHandler;

pub struct Session {
    beacon: Option<Beacon>,
    xp_receiving_address: Option<SocketAddr>,
    xp_receiving_socket: Arc<UdpSocket>,

    xp_sending_address: Option<SocketAddr>,
    xp_sending_socket: Arc<UdpSocket>,

    dataref_handler: DataRefHandler,
    command_handler: CommandHandler,
}

impl Session {
    pub fn auto_discover_default(timeout: u64) -> Result<Self, io::Error> {
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

        Ok(Session {
            beacon: Some(Beacon::new(timeout)?),
            xp_receiving_address: None,
            xp_receiving_socket: Arc::new(xp_receiving_socket),
            xp_sending_address: None,
            xp_sending_socket: Arc::new(xp_sending_socket),
            dataref_handler: DataRefHandler::new(),
            command_handler: CommandHandler::new(),
        })
    }

    pub fn auto_discover(beacon_addr: SocketAddrV4,
                         xp_receiving_address: SocketAddr,
                         xp_sending_address: SocketAddr,
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

        Ok(Session {
            beacon: Some(Beacon::new_with_address(beacon_addr, timeout)?),
            xp_receiving_address: Some(xp_receiving_address),
            xp_receiving_socket: Arc::new(xp_receiving_socket),
            xp_sending_address: Some(xp_sending_address),
            xp_sending_socket: Arc::new(xp_sending_socket),
            dataref_handler: DataRefHandler::new(),
            command_handler: CommandHandler::new(),
        })
    }

    pub fn manual(xp_receiving_address: SocketAddr,
                  xp_sending_address: SocketAddr) -> Result<Self, io::Error> {
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

        Ok(Session {
            beacon: None,
            xp_receiving_address: Some(xp_receiving_address),
            xp_receiving_socket: Arc::new(xp_receiving_socket),
            xp_sending_address: Some(xp_sending_address),
            xp_sending_socket: Arc::new(xp_sending_socket),
            dataref_handler: DataRefHandler::new(),
            command_handler: CommandHandler::new(),
        })
    }

    pub fn connect(&mut self) -> Result<(), io::Error> {
        match (self.xp_receiving_address, self.xp_sending_address) {
            // Both receiving and sending addresses provided
            // Connect to X-Plane directly
            (Some(receiving), Some(sending)) => {
                self.connect_xp(receiving, sending)?;
            },
            // Addresses not provided
            // Infer X-Plane addresses from beacon data
            _ => {
                match self.beacon {
                    Some(ref mut beacon) => {
                        // Connect to X-Plane multicast group
                        beacon.connect_beacon()?;
                        beacon.intercept_beacon()?;
                        beacon.close_beacon()?;

                        // Get beacon data
                        debug!("No X-Plane address provided, auto-discovering from beacon...");
                        let beacon_data = match beacon.get_beacon() {
                            Some(data) => data,
                            None => {
                                error!("No beacon data available, cannot auto-discover X-Plane");
                                return Err(io::Error::new(io::ErrorKind::NotFound, "No beacon data available"));
                            }
                        };

                        // Receiving address of X-Plane
                        let receiving = SocketAddr::new(
                            beacon_data.get_source().ip(),
                            beacon_data.get_port(),
                        );
                        debug!("Assuming X-Plane receiving address is {} from beacon", receiving);

                        // Sending address of X-Plane
                        let sending = SocketAddr::new(
                            beacon_data.get_source().ip(),
                            XP_DEFAULT_SENDING_PORT,
                        );
                        info!("Assuming X-Plane sending address is {}. \
                        This should work if network settings were not overridden in the simulator.", sending);

                        // Connect to X-Plane
                        self.connect_xp(receiving, sending)?;
                    },
                    None => {
                        error!("No beacon data available, cannot auto-discover X-Plane");
                        return Err(io::Error::new(io::ErrorKind::NotFound, "No beacon data available"));
                    }
                };
            }
        };
        Ok(())
    }

    fn connect_xp(&mut self, receiving: SocketAddr, sending: SocketAddr) -> Result<(), io::Error> {
        info!("Connecting to receiving side of X-Plane at {}", receiving);
        self.xp_receiving_address = Some(receiving);
        self.xp_receiving_socket.connect(receiving)?;

        info!("Connecting to sending side of X-Plane at {}", sending);
        self.xp_sending_address = Some(sending);
        self.xp_sending_socket.connect(sending)?;

        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<Beacon> {
        &self.beacon
    }

    pub fn run(&self) {
        info!("Starting receiving thread");
        self.dataref_handler.spawn_run_thread(self.xp_sending_socket.clone());
    }

    pub fn subscribe(&mut self, dataref: &str, frequency: i32, dataref_type: DataRefType) -> io::Result<()> {
        match self.xp_receiving_address {
            Some(addr) => {
                self.dataref_handler.new_subscribe(dataref, frequency, dataref_type, &self.xp_sending_socket, &addr)
            },
            None => {
                error!("Cannot subscribe to dataref without connecting to X-Plane first");
                Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected to X-Plane"))
            }
        }
    }

    pub fn unsubscribe(&mut self, dataref: &str) -> io::Result<()> {
        match self.xp_receiving_address {
            Some(addr) => {
                self.dataref_handler.unsubscribe(dataref, &self.xp_sending_socket, &addr)
            },
            None => {
                error!("Cannot unsubscribe from dataref without connecting to X-Plane first");
                Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected to X-Plane"))
            }
        }
    }

    pub fn unsubscribe_all(&mut self) -> io::Result<()> {
        match self.xp_receiving_address {
            Some(addr) => {
                self.dataref_handler.unsubscribe_all(&self.xp_sending_socket, &addr)
            },
            None => {
                error!("Cannot unsubscribe from datarefs without connecting to X-Plane first");
                Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected to X-Plane"))
            }
        }
    }

    pub fn cmd(&self, command: &str) -> io::Result<()> {
        match self.xp_receiving_address {
            Some(addr) => {
                self.command_handler.send_command(command, &self.xp_sending_socket, &addr)
            },
            None => {
                error!("Cannot send command without connecting to X-Plane first");
                Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected to X-Plane"))
            }
        }
    }

    pub fn alert(&self, message: AlertMessage) -> io::Result<()> {
        match self.xp_receiving_address {
            Some(addr) => {
                self.command_handler.alert(message, &self.xp_sending_socket, &addr)
            },
            None => {
                error!("Cannot send alert without connecting to X-Plane first");
                Err(io::Error::new(io::ErrorKind::NotConnected, "Not connected to X-Plane"))
            }
        }
    }

    pub fn get_dataref(&self, dataref: &str) -> Option<DataRefValueType> {
        self.dataref_handler.get_dataref(dataref)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Some(ref beacon) = self.beacon {
            // Close beacon, if it exists
            let _ = beacon.close_beacon();
        }

        // Unsubscribe from all datarefs
        // This tells X-Plane to actually stop sending data
        if let Err(e) = self.unsubscribe_all() {
            error!("Failed to unsubscribe from all datarefs: {}", e);
        }

        info!("Session dropped");
    }
}