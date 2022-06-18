use crate::transaction_state::TransactionState;

pub struct TransactionResponse {
    transaction_id: u64,
    transaction_state: TransactionState,
}

impl TransactionResponse {
    pub fn new(transaction_id: u64, transaction_state: TransactionState) -> Self {
        TransactionResponse {
            transaction_id,
            transaction_state,
        }
    }
}
