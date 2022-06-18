const PAYLOAD_SIZE: usize = 16;

#[derive(Debug)]
pub struct EntityPayload {
    pub transaction_id: u64,
    pub cost: u64,
}

impl EntityPayload {
    pub fn new(transaction_id: u64, cost: u64) -> Self {
        EntityPayload {
            transaction_id,
            cost,
        }
    }
}

// TODO: test
fn be_byte_buffer_to_u64(buffer: &[u8]) -> u64 {
    let mut mask_buffer = [0u8; 8];
    mask_buffer.copy_from_slice(&buffer[0..8]);
    u64::from_be_bytes(mask_buffer)
}

// TODO: test, poor's man deserialization
// se entiende en big endian
impl From<Vec<u8>> for EntityPayload {
    fn from(v: Vec<u8>) -> Self {
        if v.len() != PAYLOAD_SIZE {
            panic!();
        }
        println!("{:?}", v);
        EntityPayload {
            transaction_id: be_byte_buffer_to_u64(&v[0..8]),
            cost: be_byte_buffer_to_u64(&v[8..]),
        }
    }
}

// TODO: even more testing
impl From<EntityPayload> for Vec<u8> {
    fn from(data: EntityPayload) -> Self {
        let mut res = Vec::from(data.transaction_id.to_be_bytes());
        res.extend_from_slice(&data.cost.to_be_bytes());
        res
    }
}
