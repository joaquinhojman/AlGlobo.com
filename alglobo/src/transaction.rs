use crate::entity_data::{Entity, EntityData};
use serde::Deserialize;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum TransactionState {
    Wait,
    Commit,
    Abort,
}

#[derive(Deserialize)]
pub struct Transaction {
    id: u64,
    hotel_cost: u64,
    bank_cost: u64,
    airline_cost: u64,
}

// TODO: refactor
impl Transaction {
    pub(crate) fn get_entities_data(&self) -> Vec<(Entity, EntityData)> {
        let mut data = vec![];
        if self.hotel_cost > 0 {
            data.push((Entity::Hotel, EntityData::new(self.id, self.hotel_cost)));
        }
        if self.bank_cost > 0 {
            data.push((Entity::Bank, EntityData::new(self.id, self.bank_cost)));
        }
        if self.airline_cost > 0 {
            data.push((Entity::Airline, EntityData::new(self.id, self.airline_cost)));
        }
        data
    }
}
