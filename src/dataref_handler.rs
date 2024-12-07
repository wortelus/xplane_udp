use std::{io, thread};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use dashmap::DashMap;
use log::{debug, error, info};

use crate::consts::RREF_PREFIX;
use crate::dataref::DataRef;
use crate::dataref_type::{DataRefType, DataRefValueType};
use crate::dataref_handler::MessageStatus::InvalidData;

pub enum MessageStatus<T> {
    Ok(T),
    WrongPrefix,
    InvalidLength,
    InvalidData,
}

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
            name_id_map: DashMap::new(),
        }
    }

    pub fn should_process(data: &[u8]) -> MessageStatus<usize> {
        // Python 3 struct.pack arg: '<4xsif'
        // <: little-endian
        // 4x: 4 bytes padding
        // s: 1 byte char
        // i: 4 byte int
        // f: 4 byte float
        // ref: https://xppython3.readthedocs.io/en/latest/development/udp/rref.html

        if data.len() < 13 {
            return MessageStatus::InvalidLength;
        }

        if !data.starts_with(RREF_PREFIX) {
            return MessageStatus::WrongPrefix;
        }

        let len_no_prefix = data.len() - 5;
        match len_no_prefix % 8 {
            0 => MessageStatus::Ok(len_no_prefix / 8),
            _ => InvalidData,
        }
    }

    pub fn process_message(map: &mut Arc<DashMap<i32, DataRef>>, data: &[u8]) -> MessageStatus<usize> {
        let vars_count: usize = match DataRefHandler::should_process(data) {
            MessageStatus::Ok(e) => e,
            other => return other,
        };

        for i in 0..vars_count {
            let i_index = 5 + i * 8;
            let v_index = i_index + 4;

            let index = i32::from_le_bytes(data[i_index..v_index].try_into().unwrap());
            let value = f32::from_le_bytes(data[v_index..v_index + 4].try_into().unwrap());

            map.entry(index).and_modify(|e| e.update(value));
        }

        MessageStatus::Ok(vars_count)
    }

    pub fn spawn_run_thread(&self, receiving_socket: Arc<UdpSocket>) {
        info!("Spawning dataref handler thread");
        let mut datarefs = self.id_datarefs.clone();
        thread::spawn(move || {
            let mut buffer = [0; 4096];
            loop {
                match receiving_socket.recv(&mut buffer) {
                    Ok(received) => {
                        match DataRefHandler::process_message(&mut datarefs, &buffer[..received]) {
                            MessageStatus::Ok(count) => {
                                debug!("Processed RREF message with {} bytes and {} dataref updates", received, count);
                                continue;
                            }
                            MessageStatus::WrongPrefix => {
                                debug!("Received non-RREF data");
                                continue;
                            }
                            MessageStatus::InvalidData | MessageStatus::InvalidLength => {
                                error!("Failed to process RREF message");
                                debug!("Received data: {:?}", &buffer[..received]);
                                continue;
                            }
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

        let (_, mut dataref) = match self.id_datarefs.remove(&index) {
            Some(e) => e,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "Dataref not found")),
        };

        let message = dataref.unsubscribe_message();
        sending_socket.send_to(message.as_slice(), receiving_address)?;

        Ok(())
    }

    pub fn unsubscribe_all(&mut self, sending_socket: &UdpSocket, receiving_address: &SocketAddr) -> io::Result<()> {
        let keys: Vec<String> = self.name_id_map.iter().map(|e| e.key().clone()).collect();
        for name in keys {
            self.unsubscribe(name.as_str(), sending_socket, receiving_address)?;
        }


        self.id_datarefs.clear();
        self.name_id_map.clear();

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