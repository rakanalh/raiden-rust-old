use super::helpers::parse_address;
use web3::types::Address;

pub mod abi;

pub const CONTRACTS: &str = include_str!("data/contracts.json");
const DEPLOYMENT_KOVAN: &str = include_str!("data/deployment_kovan.json");

pub fn get_token_network_registry_address() -> Address {
    let contracts_data: serde_json::Value = serde_json::from_str(DEPLOYMENT_KOVAN).unwrap();

    let registry_address = contracts_data
        .get("contracts")
        .unwrap()
        .get("TokenNetworkRegistry")
        .unwrap()
        .get("address")
        .unwrap();

    if let Some(parsed_address) = parse_address(registry_address.as_str().unwrap().to_string()) {
        return parsed_address;
    }
    Address::zero()
}
