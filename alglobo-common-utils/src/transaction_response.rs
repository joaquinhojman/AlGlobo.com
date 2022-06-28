use crate::entity_payload::be_byte_buffer_to_u64;
use crate::transaction_state::TransactionState;

// 9 bytes de repuesta
pub const TRANSACTION_RESPONSE_PAYLOAD_SIZE: usize = 9;

#[derive(Debug)]
pub struct TransactionResponse {
    pub transaction_id: u64,
    pub transaction_state: TransactionState,
}

impl TransactionResponse {
    pub fn new(transaction_id: u64, transaction_state: TransactionState) -> Self {
        TransactionResponse {
            transaction_id,
            transaction_state,
        }
    }
}

impl From<Vec<u8>> for TransactionResponse {
    fn from(payload_buffer: Vec<u8>) -> Self {
        if payload_buffer.len() != TRANSACTION_RESPONSE_PAYLOAD_SIZE {
            panic!("Invalid buffer received when deserializing response");
        }
        let state: TransactionState = payload_buffer[0].into();
        let id = be_byte_buffer_to_u64(&payload_buffer[1..]);
        TransactionResponse::new(id, state)
    }
}

impl From<TransactionResponse> for Vec<u8> {
    fn from(response: TransactionResponse) -> Self {
        let mut res = vec![response.transaction_state.into()];
        res.extend_from_slice(response.transaction_id.to_be_bytes().as_slice());
        res
    }
}
