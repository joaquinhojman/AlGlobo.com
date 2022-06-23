const PREPARE: u8 = 0x0;
const COMMIT: u8 = 0x1;
const ABORT: u8 = 0x2;

#[derive(Debug, Copy, Clone)]
pub enum TransactionState {
    Wait, // este es estado interno, no se envia por socket
    Prepare,
    Accept, // idem wait
    Commit,
    Abort,
}

impl From<u8> for TransactionState {
    fn from(payload_byte: u8) -> Self {
        match payload_byte {
            PREPARE => TransactionState::Prepare,
            COMMIT => TransactionState::Commit,
            ABORT => TransactionState::Abort,
            _ => panic!("Could not deserialize unknown byte into state"),
        }
    }
}

impl From<TransactionState> for u8 {
    fn from(state: TransactionState) -> Self {
        match state {
            TransactionState::Prepare => PREPARE,
            TransactionState::Commit => COMMIT,
            TransactionState::Abort => ABORT,
            _ => panic!("State is not serializable"),
        }
    }
}

mod tests {
    use crate::transaction_state::TransactionState;

    #[test]
    fn test_transaction_state() {
        let mut s: TransactionState;
        s = TransactionState::from(0);
        assert_eq!(format!("{:?}", s), "Prepare");
        s = TransactionState::from(1);
        assert_eq!(format!("{:?}", s), "Commit");
        s = TransactionState::from(2);
        assert_eq!(format!("{:?}", s), "Abort");
    }
}
