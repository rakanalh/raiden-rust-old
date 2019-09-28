use crate::errors::ChannelError;
use std::collections::HashMap;
use std::rc::Rc;
use web3::types::{Address, BlockNumber, H256};

#[derive(Clone)]
pub struct CanonicalIdentifier {
    chain_identifier: u16,
    token_network_address: Address,
    channel_identifier: u64,
}

#[derive(Clone)]
pub struct ChainState {
    pub block_number: u64,
}

#[derive(Clone)]
pub struct TokenNetworkRegistryState {
    pub address: Address,
    pub token_network_list: Vec<Rc<TokenNetworkState>>,
    pub tokennetworkaddresses_to_tokennetworks: HashMap<Address, Rc<TokenNetworkState>>,
    pub tokenaddresses_to_tokennetworkaddresses: HashMap<Address, Address>,
}

impl<'a> TokenNetworkRegistryState {
    pub fn default() -> TokenNetworkRegistryState {
        TokenNetworkRegistryState {
            address: Address::zero(),
            token_network_list: vec![],
            tokennetworkaddresses_to_tokennetworks: HashMap::new(),
            tokenaddresses_to_tokennetworkaddresses: HashMap::new(),
        }
    }

    pub fn new(token_network_list: Vec<Rc<TokenNetworkState>>) -> TokenNetworkRegistryState {
        let mut registry_state = TokenNetworkRegistryState::default();
        for token_network in token_network_list.iter() {
            let token_network_address = token_network.address;
            let token_address = token_network.token_address;
            registry_state
                .tokennetworkaddresses_to_tokennetworks
                .insert(token_network_address, Rc::clone(token_network));

            registry_state
                .tokenaddresses_to_tokennetworkaddresses
                .insert(token_address, token_network.address);
        }
        registry_state
    }
}

#[derive(Clone)]
pub struct TokenNetworkState {
    address: Address,
    token_address: Address,
    network_graph: TokenNetworkGraphState,
    channelidentifiers_to_channels: HashMap<u64, ChannelState>,
    partneraddresses_to_channelidentifiers: HashMap<Address, Vec<u64>>,
}

impl TokenNetworkState {
    pub fn default() -> TokenNetworkState {
        TokenNetworkState {
            address: Address::zero(),
            token_address: Address::zero(),
            network_graph: TokenNetworkGraphState::default(),
            channelidentifiers_to_channels: HashMap::new(),
            partneraddresses_to_channelidentifiers: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct TokenNetworkGraphState {}

impl TokenNetworkGraphState {
    pub fn default() -> TokenNetworkGraphState {
        TokenNetworkGraphState {}
    }
}

#[derive(Clone)]
pub struct ChannelState {
    canonical_identifier: CanonicalIdentifier,
    token_address: Address,
    token_network_registry_address: Address,
    reveal_timeout: u16,
    settle_timeout: u16,
    our_state: OurEndState,
    partner_state: PartnerEndState,
    open_transaction: TransactionExecutionStatus,
    close_transaction: Option<TransactionExecutionStatus>,
    settle_transaction: Option<TransactionExecutionStatus>,
    update_transaction: Option<TransactionExecutionStatus>,
}

impl ChannelState {
    fn new(
        canonical_identifier: CanonicalIdentifier,
        token_address: Address,
        token_network_registry_address: Address,
        our_address: Address,
        partner_address: Address,
        reveal_timeout: u16,
        settle_timeout: u16,
        open_transaction: TransactionExecutionStatus,
    ) -> Result<ChannelState, ChannelError> {
        if reveal_timeout >= settle_timeout {
            return Err(ChannelError {
                msg: "reveal_timeout must be smaller than settle_timeout".to_string(),
            });
        }

        let our_state = OurEndState::new(our_address);
        let partner_state = PartnerEndState::new(partner_address);

        Ok(ChannelState {
            canonical_identifier,
            token_address,
            token_network_registry_address,
            reveal_timeout,
            settle_timeout,
            our_state,
            partner_state,
            open_transaction,
            close_transaction: None,
            settle_transaction: None,
            update_transaction: None,
        })
    }
}

#[derive(Clone)]
pub struct OurEndState {
    address: Address,
    contract_balance: u64,
    onchain_total_withdraw: u64,
    withdraws_pending: HashMap<u64, PendingWithdrawState>,
    withdraws_expired: Vec<ExpiredWithdrawState>,
    secrethashes_to_lockedlocks: HashMap<H256, HashTimeLockState>,
    secrethashes_to_unlockedlocks: HashMap<H256, UnlockPartialProofState>,
    secrethashes_to_onchain_unlockedlocks: HashMap<H256, UnlockPartialProofState>,
    balance_proof: Option<BalanceProofUnsignedState>,
    pending_locks: PendingLocksState,
    onchain_locksroot: H256,
    nonce: u64,
}

impl OurEndState {
    pub fn new(address: Address) -> OurEndState {
        OurEndState {
            address,
            contract_balance: 0,
            onchain_total_withdraw: 0,
            withdraws_pending: HashMap::new(),
            withdraws_expired: vec![],
            secrethashes_to_lockedlocks: HashMap::new(),
            secrethashes_to_unlockedlocks: HashMap::new(),
            secrethashes_to_onchain_unlockedlocks: HashMap::new(),
            balance_proof: None,
            pending_locks: PendingLocksState::new(),
            onchain_locksroot: H256::zero(),
            nonce: 0,
        }
    }
}

#[derive(Clone)]
pub struct PartnerEndState {
    address: Address,
    contract_balance: u64,
    onchain_total_withdraw: u64,
    withdraws_pending: HashMap<u16, PendingWithdrawState>,
    withdraws_expired: Vec<ExpiredWithdrawState>,
    secrethashes_to_lockedlocks: HashMap<H256, HashTimeLockState>,
    secrethashes_to_unlockedlocks: HashMap<H256, UnlockPartialProofState>,
    secrethashes_to_onchain_unlockedlocks: HashMap<H256, UnlockPartialProofState>,
    balance_proof: Option<BalanceProofSignedState>,
    pending_locks: PendingLocksState,
    onchain_locksroot: H256,
    nonce: u64,
}

impl PartnerEndState {
    pub fn new(address: Address) -> PartnerEndState {
        PartnerEndState {
            address,
            contract_balance: 0,
            onchain_total_withdraw: 0,
            withdraws_pending: HashMap::new(),
            withdraws_expired: vec![],
            secrethashes_to_lockedlocks: HashMap::new(),
            secrethashes_to_unlockedlocks: HashMap::new(),
            secrethashes_to_onchain_unlockedlocks: HashMap::new(),
            balance_proof: None,
            pending_locks: PendingLocksState::new(),
            onchain_locksroot: H256::zero(),
            nonce: 0,
        }
    }
}

#[derive(Clone)]
pub struct BalanceProofUnsignedState {
    nonce: u64,
    transferred_amount: u64,
    locked_amount: u64,
    locksroot: H256,
    canonical_identifier: CanonicalIdentifier,
    balance_hash: H256,
}

#[derive(Clone)]
pub struct BalanceProofSignedState {
    nonce: u64,
    transferred_amount: u64,
    locked_amount: u64,
    locksroot: H256,
    message_hash: H256,
    signature: H256,
    sender: Address,
    canonical_identifier: CanonicalIdentifier,
    balance_hash: H256,
}

#[derive(Clone)]
pub struct PendingLocksState {
    locks: Vec<H256>,
}

impl PendingLocksState {
    fn new() -> Self {
        PendingLocksState { locks: vec![] }
    }
}

#[derive(Clone)]
pub struct UnlockPartialProofState {
    lock: HashTimeLockState,
    secret: H256,
    amount: u64,
    expiration: u16,
    secrethash: H256,
    encoded: H256,
}

#[derive(Clone)]
pub struct HashTimeLockState {
    amount: u64,
    expiration: u16,
    secrethash: H256,
    encoded: H256,
}

impl HashTimeLockState {
    pub fn new(amount: u64, expiration: u16, secrethash: H256, encoded: H256) -> HashTimeLockState {
        HashTimeLockState {
            amount,
            expiration,
            secrethash,
            encoded,
        }
    }
}

#[derive(Clone)]
pub struct ExpiredWithdrawState {
    total_withdraw: u64,
    expiration: u16,
    nonce: u64,
}

#[derive(Clone)]
pub struct PendingWithdrawState {
    total_withdraw: u64,
    expiration: u16,
    nonce: u64,
}

#[derive(Clone)]
pub struct FeeScheduleState {
    flat: u64,
    proportional: u64,
    imbalance_penalty: Option<Vec<(u64, u64)>>,
    penalty_func: Option<u64>,
}

#[derive(Clone)]
pub enum TransactionResult {
    SUCCESS,
    FAILURE,
}

#[derive(Clone)]
pub struct TransactionExecutionStatus {
    started_block_number: Option<BlockNumber>,
    finished_block_number: Option<BlockNumber>,
    result: Option<TransactionResult>,
}
