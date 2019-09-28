extern crate web3;
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
    let mut service = raiden::service::RaidenService::default();

    service.start();
}
