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
}
