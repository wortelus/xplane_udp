use std::io;
use std::io::Error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use log::{debug, error, info};
use tokio::net::UdpSocket;
use tokio::runtime::Builder;
use crate::consts::XP_DEFAULT_SENDING_PORT;
use crate::beacon::Beacon;
use crate::auto_discover::AutoDiscover;
use crate::command_handler::{AlertMessage, CommandHandler};
use crate::dataref_type::{DataRefType, DataRefValueType};
use crate::dataref_handler::DataRefHandler;

pub struct Session {
    beacon: Option<Beacon>,
    xp_receiving_address: SocketAddr,
    xp_receiving_socket: Arc<UdpSocket>,

    xp_sending_address: SocketAddr,
    xp_sending_socket: Arc<UdpSocket>,

    dataref_handler: DataRefHandler,
    command_handler: CommandHandler,
}

impl Session {
    pub async fn manual(xp_receiving_address: SocketAddr,
                        xp_sending_address: SocketAddr) -> io::Result<Self> {
        let xp_receiving_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await
            .map_err(|e| {
                error!("Failed to bind to receiving socket: {}", e);
                e
            })?;
        debug!("Receiving socket bound to {}", xp_receiving_socket.local_addr()?);

        let xp_sending_socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await
            .map_err(|e| {
                error!("Failed to bind to sending socket: {}", e);
                e
            })?;
        debug!("Sending socket bound to {}", xp_sending_socket.local_addr()?);

        Ok(Session {
            beacon: None,
            xp_receiving_address,
            xp_receiving_socket: Arc::new(xp_receiving_socket),
            xp_sending_address,
            xp_sending_socket: Arc::new(xp_sending_socket),
            dataref_handler: DataRefHandler::default(),
            command_handler: CommandHandler::default(),
        })
    }

    pub async fn intercept_beacon(mut auto_discover: AutoDiscover) -> Result<Session, (Error, AutoDiscover)> {
        let beacon = auto_discover.get_beacon_mut();

        // Intercept beacon
        if let Err(err) = beacon.intercept_beacon().await {
            return Err((err, auto_discover));
        }

        // Close beacon
        if let Err(err) = beacon.close_beacon().await {
            return Err((err, auto_discover));
        }

        // Get beacon data
        debug!("No X-Plane address provided, auto-discovering from beacon...");
        let beacon_data = match beacon.get_beacon() {
            Some(data) => data,
            None => {
                error!("No beacon data available, cannot auto-discover X-Plane");
                return Err((Error::new(io::ErrorKind::NotFound, "No beacon data available"), auto_discover));
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


        let xp_receiving_socket = match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await {
            Ok(socket) => socket,
            Err(e) => {
                error!("Failed to bind to receiving socket: {}", e);
                return Err((e, auto_discover));
            }
        };

        let xp_sending_socket = match UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await {
            Ok(socket) => socket,
            Err(e) => {
                error!("Failed to bind to sending socket: {}", e);
                return Err((e, auto_discover));
            }
        };

        // Destructure AutoDiscover
        let AutoDiscover { beacon, .. } = auto_discover;

        Ok(Session {
            beacon: Some(beacon),
            xp_receiving_address: receiving,
            xp_receiving_socket: Arc::new(xp_receiving_socket),
            xp_sending_address: sending,
            xp_sending_socket: Arc::new(xp_sending_socket),
            dataref_handler: DataRefHandler::default(),
            command_handler: CommandHandler::default(),
        })
    }

    async fn connect_xp(&mut self, receiving: SocketAddr, sending: SocketAddr) -> io::Result<()> {
        info!("Connecting to receiving side of X-Plane at {}", receiving);
        self.xp_receiving_address = receiving;
        self.xp_receiving_socket.connect(receiving).await?;

        info!("Connecting to sending side of X-Plane at {}", sending);
        self.xp_sending_address = sending;
        self.xp_sending_socket.connect(sending).await?;

        Ok(())
    }

    pub fn get_beacon(&self) -> &Option<Beacon> {
        &self.beacon
    }

    pub async fn run(&mut self) -> io::Result<()> {
        info!("Connecting to X-Plane");
        self.connect_xp(self.xp_receiving_address, self.xp_sending_address).await?;

        info!("Starting receiving thread");
        self.dataref_handler.spawn_run_thread(self.xp_sending_socket.clone());
        Ok(())
    }

    pub async fn subscribe(&mut self, dataref: &str, frequency: i32, dataref_type: DataRefType) -> io::Result<()> {
        self.dataref_handler.new_subscribe(
            dataref, frequency, dataref_type, &self.xp_sending_socket, &self.xp_receiving_address)
            .await

    }

    pub async fn unsubscribe(&mut self, dataref: &str) -> io::Result<()> {
        self.dataref_handler.unsubscribe(
            dataref, &self.xp_sending_socket, &self.xp_receiving_address)
            .await
    }

    pub async fn unsubscribe_all(&mut self) -> io::Result<()> {
        self.dataref_handler.unsubscribe_all(
            &self.xp_sending_socket, &self.xp_receiving_address)
            .await
    }

    pub async fn cmd(&self, command: &str) -> io::Result<()> {
        self.command_handler.send_command(
            command, &self.xp_sending_socket, &self.xp_receiving_address)
            .await
    }

    pub async fn alert(&self, message: AlertMessage) -> io::Result<()> {
        self.command_handler.alert(
            message, &self.xp_sending_socket, &self.xp_receiving_address)
            .await
    }

    pub fn get_dataref(&self, dataref: &str) -> Option<DataRefValueType> {
        self.dataref_handler.get_dataref(dataref)
    }

    pub async fn shutdown(mut self) {
        // Close beacon, if it exists
        if let Some(ref beacon) = self.beacon {
            let _ = beacon.close_beacon();
        }

        // Unsubscribe from all datarefs
        if let Err(e) = self.unsubscribe_all().await {
            error!("Failed to unsubscribe from all datarefs: {}", e);
        }

        info!("Shutting down session");
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

        // Commented out due to async unsubscribe_all

        // if let Err(e) = self.unsubscribe_all() {
        //     error!("Failed to unsubscribe from all datarefs: {}", e);
        // }

        info!("Session dropped");
    }
}