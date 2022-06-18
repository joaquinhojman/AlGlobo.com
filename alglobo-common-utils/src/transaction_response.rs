use crate::transaction_state::TransactionState;

pub struct TransactionResponse {
    _transaction_id: u64,
    _transaction_state: TransactionState,
}

impl TransactionResponse {
    pub fn new(transaction_id: u64, transaction_state: TransactionState) -> Self {
        TransactionResponse {
            _transaction_id: transaction_id,
            _transaction_state: transaction_state,
        }
    }
}
