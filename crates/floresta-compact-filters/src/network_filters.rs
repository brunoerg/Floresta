use std::sync::PoisonError;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

use bitcoin::BlockHash;
use floresta_chain::pruned_utreexo::BlockchainInterface;

use crate::BlockFilterStore;

#[derive(Debug)]
pub struct NetworkFilters<Storage: BlockFilterStore + Send + Sync> {
    filters: Storage,
    height: RwLock<u32>,
}

impl<Storage: BlockFilterStore + Send + Sync> NetworkFilters<Storage> {
    pub fn new(filters: Storage, height: u32) -> Self {
        Self {
            filters,
            height: RwLock::new(height),
        }
    }

    pub fn get_filter(&self, height: u32) -> Option<crate::BlockFilter> {
        self.filters.get_filter(height as u64)
    }

    pub fn match_any(
        &self,
        query: Vec<&[u8]>,
        start_height: u32,
        end_height: u32,
        chain: impl BlockchainInterface,
    ) -> Vec<BlockHash> {
        let mut blocks = Vec::new();
        for height in start_height..end_height {
            let Some(filter) = self.filters.get_filter(height as u64) else {
                continue;
            };

            let mut query = query.clone().into_iter();
            let hash = chain.get_block_hash(height).unwrap();

            if filter.match_any(&hash, &mut query).unwrap() {
                let block_hash = chain.get_block_hash(height).unwrap();
                blocks.push(block_hash);
            }
        }

        blocks
    }

    pub fn push_filter(
        &self,
        height: u32,
        filter: crate::BlockFilter,
    ) -> Result<(), PoisonError<RwLockWriteGuard<u32>>> {
        self.filters.put_filter(height as u64, filter);
        self.height.write().map(|mut self_height| {
            *self_height = height;
        })
    }

    pub fn get_height(&self) -> u32 {
        self.height.read().map(|height| *height).unwrap_or(0)
    }
}
