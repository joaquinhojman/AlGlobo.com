use crate::transaction_state::TransactionState;

pub const PAYLOAD_SIZE: usize = 17;

#[derive(Debug)]
pub struct EntityPayload {
    pub transaction_state: TransactionState,
    pub transaction_id: u64,
    pub cost: u64,
}

impl EntityPayload {
    pub fn new(transaction_id: u64, cost: u64) -> Self {
        EntityPayload {
            transaction_state: TransactionState::Prepare, // si la transaccion es nueva empieza en estado prepare
            transaction_id,
            cost,
        }
    }
}

pub fn be_byte_buffer_to_u64(buffer: &[u8]) -> u64 {
    let mut mask_buffer = [0u8; 8];
    mask_buffer.copy_from_slice(&buffer[0..8]);
    u64::from_be_bytes(mask_buffer)
}

// se entiende en big endian
impl From<Vec<u8>> for EntityPayload {
    fn from(v: Vec<u8>) -> Self {
        if v.len() != PAYLOAD_SIZE {
            panic!();
        }
        EntityPayload {
            transaction_state: v[0].into(),
            transaction_id: be_byte_buffer_to_u64(&v[1..9]),
            cost: be_byte_buffer_to_u64(&v[9..]),
        }
    }
}

impl From<EntityPayload> for Vec<u8> {
    fn from(data: EntityPayload) -> Self {
        let mut res = vec![data.transaction_state.into()];
        res.extend_from_slice(&data.transaction_id.to_be_bytes());
        res.extend_from_slice(&data.cost.to_be_bytes());
        res
    }
}
