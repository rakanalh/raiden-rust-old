use crate::enums::Event;
use crate::service::RaidenService;

pub struct EventHandler {}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }

    pub async fn handle_event(&self, raiden: &RaidenService, event: Event) {
        match event {
            Event::TokenNetworkCreated(event) => {
                let token_network_address = event.token_network.address;
                raiden
                    .contracts_registry
                    .create_contract_event_filters("TokenNetwork".to_string(), token_network_address);
                raiden.poll_filters().await;
            }
        }
    }
}
