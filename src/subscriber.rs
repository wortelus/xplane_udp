use std::{io, thread};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use dashmap::DashMap;
use log::{debug, error, info};

use crate::consts::RREF_PREFIX;
use crate::dataref::{DataRef, DataRefType, DataRefValueType};

pub struct DataRefHandler {
    index_counter: i32,
    id_datarefs: Arc<DashMap<i32, DataRef>>,
    name_id_map: DashMap<String, i32>,
}

impl DataRefHandler {
    pub fn new() -> Self {
        DataRefHandler {
            index_counter: 1,
            id_datarefs: Arc::new(DashMap::new()),
            name_id_map: DashMap::new()
        }
    }

    pub fn should_process(data: &[u8; 4096]) -> bool {
        // match data[0..4] {
        //     RREF_PREFIX => true,
        //     _ => false,
        // }
        data.starts_with(RREF_PREFIX)
    }

    pub fn process_message(map: &mut Arc<DashMap<i32, DataRef>>, data: &[u8]) -> bool {
        // Python 3 struct.pack arg: '<4xsif'
        // <: little-endian
        // 4x: 4 bytes padding
        // s: 1 byte char
        // i: 4 byte int
        // f: 4 byte float
        // ref: https://xppython3.readthedocs.io/en/latest/development/udp/rref.html
        if data.len() < 13 {
            return false;
        }

        if !data.starts_with(RREF_PREFIX) {
            return false;
        }

        let index = i32::from_le_bytes(data[5..9].try_into().unwrap());
        let value = f32::from_le_bytes(data[9..13].try_into().unwrap());

        map.entry(index).and_modify(|e| e.update(value));

        true
    }

    pub fn spawn_run_thread(&self, receiving_socket: Arc<UdpSocket>) {
        info!("Spawning dataref handler thread");
        let mut datarefs = self.id_datarefs.clone();
        thread::spawn(move || {
            let mut buffer = [0; 4096];
            // TODO: remove
            // debug!("{:?}", receiving_socket.local_addr());
            // debug!("{:?}", receiving_socket.peer_addr().unwrap());
            // debug!("{:?}", receiving_socket.recv(&mut buffer).unwrap());
            loop {
                match receiving_socket.recv(&mut buffer) {
                    Ok(received) => {
                        if !DataRefHandler::should_process(&buffer) {
                            continue;
                        }

                        debug!("Received RREF message with {} bytes", received);
                        if !DataRefHandler::process_message(&mut datarefs, &buffer[..received]) {
                            error!("Failed to process RREF message");
                            debug!("Received data: {:?}", &buffer[..received]);
                            continue;
                        }
                    }
                    Err(e) => {
                        error!("Error receiving dataref: {}", e);
                    }
                }
            }
        });
    }

    pub fn new_subscribe(&mut self, name: &str, frequency: i32, dataref_type: DataRefType,
                         sending_socket: &UdpSocket, receiving_address: &SocketAddr) -> io::Result<()> {
        // TODO: smarter index counter
        let index = self.index_counter;
        self.index_counter += 1;

        let dataref = DataRef::new(name, index, frequency, dataref_type);
        sending_socket.send_to(dataref.subscription_message().as_slice(), receiving_address)?;

        self.id_datarefs.insert(dataref.get_index(), dataref);
        self.name_id_map.insert(name.to_string(), index);

        Ok(())
    }

    pub fn unsubscribe(&mut self, dataref: &str,
                       sending_socket: &UdpSocket, receiving_address: &SocketAddr) -> io::Result<()> {
        let index = match self.name_id_map.get(dataref) {
            Some(e) => *e,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Dataref not found")),
        };

        let (_, dataref) = match self.id_datarefs.remove(&index) {
            Some(e) => e,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Dataref not found")),
        };

        let message = dataref.unsubscribe_message();
        sending_socket.send_to(message.as_slice(), receiving_address)?;

        Ok(())
    }

    pub fn get_dataref(&self, dataref: &str) -> Option<DataRefValueType> {
        let index = match self.name_id_map.get(dataref) {
            Some(e) => *e,
            None => return None,
        };
        self.id_datarefs.get(&index).map(|e| e.get())
    }
}