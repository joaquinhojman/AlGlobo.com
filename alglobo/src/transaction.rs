use crate::entity_data::{Entity, EntityData};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Transaction {
    id: u64,
    hotel_cost: u64,
    bank_cost: u64,
    airline_cost: u64,
}

// TODO: refactor
impl Transaction {
    pub(crate) fn get_entities_data(&self) -> Vec<EntityData> {
        let mut data = vec![];
        if self.hotel_cost > 0 {
            data.push(EntityData::new(self.id, Entity::Hotel, self.hotel_cost));
        }
        if self.bank_cost > 0 {
            data.push(EntityData::new(self.id, Entity::Bank, self.bank_cost));
        }
        if self.airline_cost > 0 {
            data.push(EntityData::new(self.id, Entity::Airline, self.airline_cost));
        }
        data
    }
}
