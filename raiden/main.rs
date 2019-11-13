extern crate tokio_core;
extern crate web3;

use raiden::accounts::keystore;
use raiden::api::http;
use raiden::cli;
use raiden::service;
use raiden::utils::{ToHTTPEndpoint, ToSocketEndpoint};
use std::path::Path;

fn main() {
    let cli_app = cli::get_cli_app();
    let matches = cli_app.get_matches();

    let chain_name = matches.value_of("chain-id").unwrap();
    let chain_id = chain_name.parse().unwrap();

    let eth_rpc_endpoint = matches.value_of("eth-rpc-endpoint").unwrap();

    let keystore_path = Path::new(matches.value_of("keystore-path").unwrap());
    let keys = keystore::list_keys(keystore_path).unwrap();

    let selected_key_filename = cli::prompt_key(&keys);
    let our_address = keys[&selected_key_filename].clone();
    let private_key = cli::prompt_password(selected_key_filename);

    let http_endpoint = eth_rpc_endpoint.to_http();
    if let Err(e) = http_endpoint {
        println!("Invalid RPC endpoint: {}", e);
        return
    }

    let socket_endpoint = eth_rpc_endpoint.to_socket();
    if let Err(e) = socket_endpoint {
        println!("Invalid RPC endpoint: {}", e);
        return
    }

    let config = cli::Config {
        keystore_path: keystore_path,
        private_key: private_key,
        eth_http_rpc_endpoint: http_endpoint.unwrap(),
        eth_socket_rpc_endpoint: socket_endpoint.unwrap(),
    };

    let mut eloop = tokio_core::reactor::Core::new().unwrap();

    let service = service::RaidenService::new(chain_id, our_address, config.private_key);
    service.initialize(config.eth_http_rpc_endpoint);
    service.start(&eloop.handle());

    if let Some(_) = matches.subcommand_matches("run") {
        let server = http::server();
        let _ = eloop.run(server);
    }
}
