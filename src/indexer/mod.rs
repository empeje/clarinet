pub mod chains;

use crate::types::{BitcoinBlockData, BlockIdentifier, StacksBlockData, StacksBlockMetadata};
use crate::utils::stacks::PoxInfo;
use rocket::serde::json::Value as JsonValue;
use std::collections::{HashMap, VecDeque};

#[allow(dead_code)]
pub enum BitcoinChainEvent {
    ChainUpdatedWithBlock(BitcoinBlockData),
    ChainUpdatedWithReorg(Vec<BitcoinBlockData>),
}

#[allow(dead_code)]
pub enum StacksChainEvent {
    ChainUpdatedWithBlock(StacksBlockData),
    ChainUpdatedWithReorg(Vec<StacksBlockData>),
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct AssetClassCache {
    pub symbol: String,
    pub decimals: u8,
}

pub struct StacksChainContext {
    asset_class_map: HashMap<String, AssetClassCache>,
    pox_info: PoxInfo,
}

impl PoxInfo {
    pub fn default() -> PoxInfo {
        PoxInfo {
            contract_id: "ST000000000000000000002AMW42H.pox".into(),
            pox_activation_threshold_ustx: 0,
            first_burnchain_block_height: 100,
            prepare_phase_block_length: 1,
            reward_phase_block_length: 4,
            reward_slots: 8,
            total_liquid_supply_ustx: 1000000000000000,
            ..Default::default()
        }
    }
}

impl StacksChainContext {
    pub fn new() -> StacksChainContext {
        StacksChainContext {
            asset_class_map: HashMap::new(),
            pox_info: PoxInfo::default(),
        }
    }
}

pub struct IndexerConfig {
    pub stacks_node_rpc_url: String,
    pub bitcoin_node_rpc_url: String,
    pub bitcoin_node_rpc_username: String,
    pub bitcoin_node_rpc_password: String,
}

pub struct Indexer {
    config: IndexerConfig,
    stacks_last_7_blocks: VecDeque<(BlockIdentifier, StacksBlockMetadata)>,
    bitcoin_last_7_blocks: VecDeque<BlockIdentifier>,
    pub stacks_context: StacksChainContext,
}

impl Indexer {
    pub fn new(config: IndexerConfig) -> Indexer {
        let stacks_last_7_blocks = VecDeque::new();
        let bitcoin_last_7_blocks = VecDeque::new();
        let stacks_context = StacksChainContext::new();
        Indexer {
            config,
            stacks_last_7_blocks,
            bitcoin_last_7_blocks,
            stacks_context,
        }
    }

    pub fn handle_bitcoin_block(&mut self, marshalled_block: JsonValue) -> BitcoinChainEvent {
        let block = chains::standardize_bitcoin_block(&self.config, marshalled_block);
        if let Some(tip) = self.bitcoin_last_7_blocks.back() {
            if block.block_identifier.index == tip.index + 1 {
                self.bitcoin_last_7_blocks
                    .push_back(block.block_identifier.clone());
                if self.bitcoin_last_7_blocks.len() > 7 {
                    self.bitcoin_last_7_blocks.pop_front();
                }
            } else if block.block_identifier.index > tip.index + 1 {
                // TODO: we received a block and we don't have the parent
            } else if block.block_identifier.index == tip.index {
                // TODO: 1 block reorg
            } else {
                // TODO: deeper reorg
            }
        } else {
            self.bitcoin_last_7_blocks
                .push_front(block.block_identifier.clone());
        }
        BitcoinChainEvent::ChainUpdatedWithBlock(block)
    }

    pub fn handle_stacks_block(&mut self, marshalled_block: JsonValue) -> StacksChainEvent {
        let block = chains::standardize_stacks_block(
            &self.config,
            marshalled_block,
            &mut self.stacks_context,
        );
        if let Some((tip, _)) = self.stacks_last_7_blocks.back() {
            if block.block_identifier.index == tip.index + 1 {
                self.stacks_last_7_blocks
                    .push_back((block.block_identifier.clone(), block.metadata.clone()));
                if self.stacks_last_7_blocks.len() > 7 {
                    self.stacks_last_7_blocks.pop_front();
                }
            } else if block.block_identifier.index > tip.index + 1 {
                // TODO: we received a block and we don't have the parent
            } else if block.block_identifier.index == tip.index {
                // TODO: 1 block reorg
            } else {
                // TODO: deeper reorg
            }
        } else {
            self.stacks_last_7_blocks
                .push_front((block.block_identifier.clone(), block.metadata.clone()));
        }
        StacksChainEvent::ChainUpdatedWithBlock(block)
    }

    pub fn get_pox_info(&mut self) -> PoxInfo {
        self.stacks_context.pox_info.clone()
    }
}
