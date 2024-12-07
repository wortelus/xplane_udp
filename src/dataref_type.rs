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

impl PartialEq for DataRefValueType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DataRefValueType::Float(a), DataRefValueType::Float(b)) => a == b,
            (DataRefValueType::Int(a), DataRefValueType::Int(b)) => a == b,
            (DataRefValueType::Char(a), DataRefValueType::Char(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for DataRefValueType {}

impl std::fmt::Debug for DataRefValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataRefValueType::Float(v) => write!(f, "Float({})", v),
            DataRefValueType::Int(v) => write!(f, "Int({})", v),
            DataRefValueType::Char(v) => write!(f, "Char({})", v),
        }
    }
}