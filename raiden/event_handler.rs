use crate::enums::Event;
use crate::service::RaidenService;

pub struct EventHandler {}

impl EventHandler {
    pub async fn handle_event(raiden: &RaidenService, event: Event) {
        match event {
            Event::TokenNetworkCreated(event) => {
                let token_network_address = event.token_network.address;
                raiden.contracts_registry.create_contract_event_filters(
                    "TokenNetwork".to_string(),
                    token_network_address.into(),
                    event.block_number.into(),
                );
                raiden.poll_filters().await;
            }
        }
    }
}
