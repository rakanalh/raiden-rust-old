use ethabi;
use serde_json;
use std::cell::RefCell;
use std::collections::HashMap;
use web3::types::{Address, BlockNumber, Filter, FilterBuilder, Log, H256, U64};

#[derive(Clone, Debug)]
pub struct Event {
    pub name: String,
    pub block_number: U64,
    pub block_hash: H256,
    pub transaction_hash: H256,
    pub data: Vec<ethabi::Token>,
}

#[derive(Default)]
pub struct ContractRegistry {
    contracts: HashMap<String, ethabi::Contract>,
    pub filters: RefCell<HashMap<String, HashMap<String, Filter>>>,
}

impl ContractRegistry {
    pub fn default() -> ContractRegistry {
        let mut registry = ContractRegistry {
            contracts: HashMap::new(),
            filters: RefCell::new(HashMap::new()),
        };

        let contracts_map: serde_json::Value = serde_json::from_str(super::CONTRACTS).unwrap();
        let objects = contracts_map.get("contracts").unwrap().as_object().unwrap();
        for (contract_name, value) in objects.iter() {
            let result = ContractRegistry::load_contract_from_abi_string(
                serde_json::to_string(value.get("abi").unwrap()).unwrap(),
            );
            if let Ok(contract) = result {
                registry.contracts.insert(contract_name.clone(), contract);
            }
        }
        registry
    }

    pub fn create_contract_event_filters(&self, contract_name: String, contract_address: Address) {
        let mut contracts_map = self.filters.borrow_mut();
        if contracts_map.get(&contract_name).is_none() {
            for (name, contract) in &self.contracts {
                if name.as_str() != contract_name.as_str() {
                    continue;
                }

                contracts_map.insert(contract_name.clone(), HashMap::new());
                let filters = contracts_map.get_mut(&contract_name).unwrap();
                let events = contract.events();
                for event in events {
                    let event_sig = event.signature();
                    let filter = FilterBuilder::default()
                        .address(vec![contract_address])
                        .topics(Some(vec![event_sig]), None, None, None)
                        .from_block(BlockNumber::Earliest)
                        .to_block(BlockNumber::Latest)
                        .build();
                    filters.insert(event.name.clone(), filter);
                }
            }
        }
    }

    pub fn log_to_event(&self, log: &Log) -> Option<Event> {
        for contract in self.contracts.values() {
            let events = contract.events();
            for event in events {
                if !log.topics.is_empty() && event.signature() == log.topics[0] {
                    let non_indexed_inputs: Vec<ethabi::ParamType> = event
                        .inputs
                        .iter()
                        .filter(|input| !input.indexed)
                        .map(|input| input.kind.clone())
                        .collect();
                    let mut data: Vec<ethabi::Token> = vec![];

                    if log.topics.len() >= 2 {
                        let indexed_inputs: Vec<&ethabi::EventParam> =
                            event.inputs.iter().filter(|input| input.indexed).collect();
                        for topic in &log.topics[1..] {
                            if let Ok(decoded_value) =
                                ethabi::decode(&[indexed_inputs[0].kind.clone()], &topic.0)
                            {
                                data.push(decoded_value[0].clone());
                            }
                        }
                    }

                    if !log.data.0.is_empty() {
                        data.extend(ethabi::decode(&non_indexed_inputs, &log.data.0).unwrap());
                    }

                    return Some(Event {
                        name: event.name.clone(),
                        block_number: log.block_number.unwrap(),
                        block_hash: log.block_hash.unwrap(),
                        transaction_hash: log.transaction_hash.unwrap(),
                        data,
                    });
                }
            }
        }
        None
    }

    pub fn load_contract_from_abi_string(abi: String) -> Result<ethabi::Contract, ethabi::Error> {
        ethabi::Contract::load(abi.as_bytes())
    }
}
