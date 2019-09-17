use web3::types::BlockNumber;


pub struct Block {
    pub chain_id: u32,
    pub block_number: BlockNumber,
}

impl Block {
    pub fn new(chain_id: u32, block_number: BlockNumber) -> Block {
        Block {
            chain_id,
            block_number
        }
    }
}
