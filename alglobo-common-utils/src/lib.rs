extern crate core;

pub mod entity_payload;
pub mod entity_type;
pub mod transaction_request;
pub mod transaction_response;
pub mod transaction_state;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
