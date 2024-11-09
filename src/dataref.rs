use crate::consts::RREF_PREFIX;

pub enum DataRefType {
    Float,
    Int,
    Char,
}

pub enum DataRefValueType {
    Float(f32),
    Int(i32),
    Char(char),
}

pub struct DataRef {
    name: String,
    index: i32,
    freq: i32,

    value_type: DataRefType,
    raw: f32,
}

impl DataRef {
    pub fn new(name: &str, index: i32, frequency: i32, value_type: DataRefType) -> DataRef {
        let name = name.to_string();
        DataRef {
            name,
            index,
            value_type,
            freq: frequency,
            raw: 0.0,
        }
    }

    pub fn get_raw(&self) -> f32 {
        self.raw
    }

    pub fn get(&self) -> DataRefValueType {
        match self.value_type {
            DataRefType::Float => DataRefValueType::Float(self.raw),
            DataRefType::Int => DataRefValueType::Int(self.raw as i32),
            DataRefType::Char => DataRefValueType::Char(self.raw as u8 as char),
        }
    }

    pub fn update(&mut self, value: f32) {
        self.raw = value;
    }

    pub fn subscription_message(&self) -> Vec<u8> {
        // Python 3 struct.pack arg: '<4sxii400s'
        // <: little-endian
        // 4s: 4 byte string
        // x: pad byte
        // i: 4 byte int
        // i: 4 byte int
        // 400s: 400 byte string
        // ref: https://xppython3.readthedocs.io/en/latest/development/udp/rref.html
        let name_len = self.name.len();
        let len = 4 + 1 + 4 + 4;
        let max_name_len = 400;
        let total_len = max_name_len + len;
        let mut message = vec![0; total_len];

        message[0..4].copy_from_slice(RREF_PREFIX);
        // frequency to update the dataref (times per second)
        message[5..9].copy_from_slice(&self.freq.to_le_bytes());
        // index is used to identify the dataref within the communication
        message[9..13].copy_from_slice(&self.index.to_le_bytes());
        // dataref string
        message[13..13+name_len].copy_from_slice(self.name.as_bytes());

        message
    }

    pub fn unsubscribe_message(&self) -> Vec<u8> {
        // Python 3 struct.pack arg: '<4sxii400s'
        // <: little-endian
        // 4s: 4 byte string
        // x: pad byte
        // i: 4 byte int
        // i: 4 byte int
        // 400s: 400 byte string
        // ref: https://xppython3.readthedocs.io/en/latest/development/udp/rref.html
        let len = 4 + 1 + 4 + 4 + self.name.len() + 1; // + 1 for null terminator
        let mut message = vec![0; len];

        message[0..4].copy_from_slice(RREF_PREFIX);
        // index is used to identify the dataref within the communication
        message[5..9].copy_from_slice(&self.index.to_le_bytes());
        // set frequency to 0 to unsubscribe
        message[10..14].copy_from_slice(&0i32.to_le_bytes());
        // dataref string
        message[15..].copy_from_slice(self.name.as_bytes());

        message
    }

    pub fn get_name(&self) -> &str { &self.name }
    pub fn get_index(&self) -> i32 { self.index }
    pub fn get_freq(&self) -> i32 { self.freq }
    pub fn get_value_type(&self) -> &DataRefType { &self.value_type }
}