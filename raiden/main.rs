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

use web3::futures::{Future, Stream};


use raiden;
use raiden::transfer;

fn main() {
    let (_eloop, ws) = web3::transports::WebSocket::new("ws://47.100.34.71:8547").unwrap();
    let web3 = web3::Web3::new(ws.clone());

    let mut sub = web3.eth_subscribe().subscribe_new_heads().wait().unwrap();

    let manager = raiden::state::StateManager::new();

    (&mut sub).for_each(|block| {
        let block_number = web3::types::BlockNumber::Number(block.number.unwrap());
        let block_state_change = transfer::state_change::Block::new(1, block_number);


        manager.dispatch(Box::new(block_state_change))?;

        Ok(())
    }).wait().unwrap();

    sub.unsubscribe();

    drop(web3);
}
