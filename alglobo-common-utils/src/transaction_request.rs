use crate::entity_payload::EntityPayload;
use crate::entity_type::EntityType;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransactionRequest {
    id: u64,
    hotel_cost: u64,
    bank_cost: u64,
    airline_cost: u64,
}

// TODO: refactor
impl TransactionRequest {
    pub fn get_entities_data(&self) -> Vec<(EntityType, EntityPayload)> {
        let mut data = vec![];
        if self.hotel_cost > 0 {
            data.push((
                EntityType::Hotel,
                EntityPayload::new(self.id, self.hotel_cost),
            ));
        }
        if self.bank_cost > 0 {
            data.push((
                EntityType::Bank,
                EntityPayload::new(self.id, self.bank_cost),
            ));
        }
        if self.airline_cost > 0 {
            data.push((
                EntityType::Airline,
                EntityPayload::new(self.id, self.airline_cost),
            ));
        }
        data
    }

    pub fn get_transaction_id(&self) -> u64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionRequest;

    #[test]
    fn test_get_entities_data() {
        let t_hotel = TransactionRequest {
            id: 0,
            hotel_cost: 10,
            bank_cost: 0,
            airline_cost: 0,
        };
        let data = &t_hotel.get_entities_data()[0];
        assert_eq!(format!("{:?}", data.0), "Hotel");
        assert_eq!(data.1.cost, 10);
        assert_eq!(data.1.transaction_id, 0);

        let t_banco = TransactionRequest {
            id: 1,
            hotel_cost: 0,
            bank_cost: 10,
            airline_cost: 0,
        };
        let data = &t_banco.get_entities_data()[0];
        assert_eq!(format!("{:?}", data.0), "Bank");
        assert_eq!(data.1.cost, 10);
        assert_eq!(data.1.transaction_id, 1);

        let t_airline = TransactionRequest {
            id: 2,
            hotel_cost: 0,
            bank_cost: 0,
            airline_cost: 10,
        };
        let data = &t_airline.get_entities_data()[0];
        assert_eq!(format!("{:?}", data.0), "Airline");
        assert_eq!(data.1.cost, 10);
        assert_eq!(data.1.transaction_id, 2);
    }
}
