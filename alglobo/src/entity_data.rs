#[derive(Eq, Hash, PartialEq, Debug, Copy, Clone)]
pub enum Entity {
    Hotel,
    Bank,
    Airline,
}

#[derive(Debug)]
pub struct EntityData {
    pub transaction_id: u64,
    pub entity_type: Entity,
    pub cost: u64,
}

impl EntityData {
    pub fn new(transaction_id: u64, entity_type: Entity, cost: u64) -> Self {
        EntityData {
            transaction_id,
            entity_type,
            cost,
        }
    }
}
