use crate::blockchain::contracts::abi::ContractRegistry;
use crate::blockchain::contracts::abi::Event;
use crate::enums::StateChange;
use crate::transfer::state::{CanonicalIdentifier, ChannelState, TokenNetworkState, TransactionExecutionStatus, TransactionResult};
use crate::transfer::state_change::{
    ContractReceiveChannelOpened, ContractReceiveTokenNetworkCreated,
};
use ethabi::Token;
use web3::types::{Address, Log, U256};

fn create_token_network_created_state_change(base_event: Event, log: &Log) -> StateChange {
    let token_address = match base_event.data[0] {
        Token::Address(address) => address,
        _ => Address::zero(),
    };
    let token_network_address = match base_event.data[1] {
        Token::Address(address) => address,
        _ => Address::zero(),
    };
    let token_network = TokenNetworkState::new(token_network_address, token_address);
    let token_network_registry_address = log.address;
    StateChange::ContractReceiveTokenNetworkCreated(ContractReceiveTokenNetworkCreated {
        transaction_hash: Some(base_event.transaction_hash),
        block_number: base_event.block_number,
        block_hash: base_event.block_hash,
        token_network_registry_address,
        token_network,
    })
}

fn create_channel_opened_state_change(base_event: Event, log: &Log) -> StateChange {
    let channel_identifier = match base_event.data[0] {
        Token::Uint(identifier) => identifier,
        _ => U256::zero(),
    };
    let participant1 = match base_event.data[1] {
        Token::Address(address) => address,
        _ => Address::zero(),
    };
    let participant2 = match base_event.data[2] {
        Token::Address(address) => address,
        _ => Address::zero(),
    };
    let settle_timeout = match base_event.data[3] {
        Token::Uint(timeout) => timeout,
        _ => U256::zero(),
    };

    let chain_identifier = 1;
    let token_network_address = log.address;
    let token_address = Address::zero();
    let token_network_registry_address = Address::zero();
    let our_address = Address::zero();
    let partner_address = Address::zero();
    let reveal_timeout = 0;
    let settle_timeout = 0;
    let open_transaction = TransactionExecutionStatus{
        started_block_number: Some(0),
        finished_block_number: Some(0),
        result: Some(TransactionResult::SUCCESS)
    };
    let channel_state = ChannelState::new(
        CanonicalIdentifier {
            chain_identifier,
            token_network_address,
            channel_identifier,
        },
        token_address,
        token_network_registry_address,
        our_address,
        partner_address,
        reveal_timeout,
        settle_timeout,
        open_transaction,
    );

    StateChange::ContractReceiveChannelOpened(ContractReceiveChannelOpened {
        transaction_hash: Some(base_event.transaction_hash),
        block_number: base_event.block_number,
        block_hash: base_event.block_hash,
        channel_state: channel_state.unwrap()
    })
}

pub fn log_to_blockchain_state_change(
    contract_registry: &ContractRegistry,
    log: &Log,
) -> Option<StateChange> {
    let base_event = contract_registry.log_to_event(log)?;

    match base_event.name.as_ref() {
        "TokenNetworkCreated" => Some(create_token_network_created_state_change(base_event, log)),
        "ChannelOpened" => Some(create_channel_opened_state_change(base_event, log)),
        &_ => None,
    }
}
