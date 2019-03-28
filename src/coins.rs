use crate::incremental_tree::tree::SaplingMerkleTree;
use crate::key::key_management::FrHash;
use crate::transaction::Transaction;
use std::collections::hash_map::HashMap;

use bigint::U256;

pub struct AnchorsSaplingCacheEntry {
    entered: bool,
    tree: SaplingMerkleTree,
    flags: char,
}

impl AnchorsSaplingCacheEntry {
    fn new(tree: SaplingMerkleTree) -> Self {
        AnchorsSaplingCacheEntry {
            entered: false,
            flags: char::from(0),
            tree: tree,
        }
    }
}

struct NullifiersCacheEntry {
    entered: bool,
    dirty: bool,
}

impl NullifiersCacheEntry {
    fn new() -> Self {
        NullifiersCacheEntry {
            entered: false,
            dirty: false,
        }
    }
}

type AnchorsSaplingMap = HashMap<FrHash, AnchorsSaplingCacheEntry>;
type NullifiersMap = HashMap<U256, NullifiersCacheEntry>;

pub trait CoinsView {
    fn get_best_anchor(&self) -> Option<FrHash>;

    //Retrieve the tree (Sapling) at a particular anchored root in the chain
    fn get_sapling_anchor_at(&mut self, rt: FrHash) -> Option<SaplingMerkleTree>;

    //Determine whether a nullifier is spent or not
    fn get_nullifier(&mut self, nullifier: FrHash) -> bool;

    fn push_anchor(&mut self, tree: SaplingMerkleTree);

    fn set_best_block(&mut self, block_hash: U256);
}

//
pub struct CoinViewDB {}

impl CoinViewDB {
    pub fn new() -> Self {
        CoinViewDB {}
    }
}

//TODO, not necessary now
impl CoinsView for CoinViewDB {
    fn get_best_anchor(&self) -> Option<FrHash> {
        None
    }

    fn get_sapling_anchor_at(&mut self, rt: FrHash) -> Option<SaplingMerkleTree> {
        None
    }

    fn get_nullifier(&mut self, nullifier: FrHash) -> bool {
        false
    }

    fn push_anchor(&mut self, tree: SaplingMerkleTree) {}

    fn set_best_block(&mut self, block_hash: U256) {}
}

pub struct CoinViewCache {
    //mutable uint256 hashSaplingAnchor;
    hash_sapling_anchor: Option<FrHash>,
    cached_sapling_anchors: AnchorsSaplingMap,
    cached_sapling_nullifiers: NullifiersMap,
    base: CoinViewDB,
}

impl CoinViewCache {
    pub fn new() -> Self {
        CoinViewCache {
            hash_sapling_anchor: None,
            cached_sapling_anchors: AnchorsSaplingMap::new(),
            cached_sapling_nullifiers: NullifiersMap::new(),
            base: CoinViewDB::new(),
        }
    }

    pub fn set_nullifiers(&mut self, tx: Transaction, spent: bool) {
        for spend_description in tx.v_shielded_spend {
            let mut entry = NullifiersCacheEntry::new();
            entry.entered = spent;
            entry.dirty = true;
            //TODO
            //let nullifier = U256::from(spend_description.nullifier);
            //self.cached_sapling_nullifiers.insert(nullifier, entry);
        }
    }
}

impl CoinsView for CoinViewCache {
    fn get_best_anchor(&self) -> Option<FrHash> {
        self.hash_sapling_anchor
    }

    //bool CCoinsViewCache::GetSaplingAnchorAt(const uint256 &rt, SaplingMerkleTree &tree) const {
    fn get_sapling_anchor_at(&mut self, rt: FrHash) -> Option<SaplingMerkleTree> {
        let res = self.cached_sapling_anchors.get(&rt);
        match res {
            None => {
                return None;
            }
            Some(ref entry) => {
                if entry.entered {
                    return Some(entry.tree.clone());
                }
            }
        }

        let tree = self.base.get_sapling_anchor_at(rt.clone());
        let tree_clone = tree.clone();
        match tree_clone {
            None => {
                return None;
            }
            Some(t) => {
                let entry = AnchorsSaplingCacheEntry::new(t);
                let rt_clone = rt.clone();
                self.cached_sapling_anchors.insert(rt_clone, entry);
                tree
            }
        }

        //Some(tree.unwrap())
    }

    fn get_nullifier(&mut self, nullifier: FrHash) -> bool {
        false
    }

    fn push_anchor(&mut self, tree: SaplingMerkleTree) {}

    fn set_best_block(&mut self, block_hash: U256) {}
}
