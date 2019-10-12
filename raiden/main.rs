extern crate web3;
extern crate clap;

use clap::{Arg, App, SubCommand};

/*
use web3::futures::Future;
use web3::types::BlockNumber;

fn main() {
    let (_eloop, transport) = web3::transports::Http::new("http://parity.goerli.ethnodes.brainbot.com:8545").unwrap();
    let web3 = web3::Web3::new(transport);

    loop {
        let block = web3.eth().block_with_txs(BlockNumber::Latest.into()).wait().unwrap();

        println!("Latest block is: {:?}", block.unwrap().number);
    }
}
 */

use raiden;

fn main() {
    let matches = App::new("My Super Program")
        .arg(Arg::with_name("chain-id")
             .short("c")
             .long("chain-id")
             .possible_values(&["ropsten", "kovan", "goerli", "mainnet"])
             .default_value("mainnet")
             .required(true)
             .takes_value(true)
             .help("Specify the blockchain to run Raiden with"))
        .arg(Arg::with_name("eth-rpc-endpoint")
             .short("e")
             .long("eth-rpc-endpoint")
             .required(true)
             .help("Specify the RPC endpoint to interact with"))
        .arg(Arg::with_name("verbosity")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("run")
                    .about("Run the raiden client"))
        .get_matches();

    let chain_name = matches.value_of("chain-id").unwrap();
    let chain_id = chain_name.parse().unwrap();

    if let Some(_run_matches) = matches.subcommand_matches("run") {
        let mut service = raiden::service::RaidenService::new(
            chain_id,
        );

        service.start();
    }
}
