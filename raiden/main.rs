extern crate tokio_core;
extern crate web3;

use raiden::accounts::keystore;
use raiden::api::http;
use raiden::cli;
use raiden::service;
use std::path::Path;

fn main() {
    let cli_app = cli::get_cli_app();
    let matches = cli_app.get_matches();

    let chain_name = matches.value_of("chain-id").unwrap();
    let chain_id = chain_name.parse().unwrap();

    let keystore_path = Path::new(matches.value_of("keystore-path").unwrap());
    let keys = keystore::list_keys(keystore_path).unwrap();

    let selected_key_filename = cli::prompt_key(&keys);
    let our_address = keys[&selected_key_filename].clone();
    let secret_key = cli::prompt_password(selected_key_filename);

    let mut eloop = tokio_core::reactor::Core::new().unwrap();

    if let Some(_) = matches.subcommand_matches("run") {
        let service = service::RaidenService::new(chain_id, our_address, secret_key);

        service.start(&eloop);
        let server = http::server();
        eloop.run(server);
    }
}
