use ethereum_types::U256;
use ff::PrimeField;
use pairing::bls12_381::{Bls12, Fr, FrRepr};
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use crate::block_chain::{Block, BlockIndex, Chain};
use crate::coins::{CoinViewCache, CoinsView};
use crate::incremental_tree::tree::{SaplingMerkleTree, SaplingWitness};
use crate::key::key_management::{
    FrHash, SaplingExtendedFullViewingKey, SaplingExtendedSpendingKey, SaplingIncomingViewingKey,
    SaplingOutputDescription, SaplingPaymentAddress,
};
use crate::key::key_store::KeyStore;
use crate::main_impl::read_block_from_disk;
use crate::my::constants::WITNESS_CACHE_SIZE;
use crate::sendmany::SaplingOutPoint;
use crate::transaction::NoteDataMap;
use crate::transaction::{Transaction, WalletTransaction};

type SaplingIncomingViewingKeyMap = HashMap<SaplingPaymentAddress, SaplingIncomingViewingKey>;
//type TxSpendMap =

pub struct Wallet<'a> {
    pub map_wallet: HashMap<FrHash, WalletTransaction>,
    //pub map_tx_sapling_nullifier:
    nWitnessCacheSize: usize,
    n_time_first_key: i64,

    pub chain_active: &'a Chain,
    pub pcoins_tip: &'a mut CoinViewCache,

    key_store: KeyStore,
    seed: [u8; 32],
}

impl<'a> Wallet<'a> {
    pub fn new(pcoins_tip: &'a mut CoinViewCache, chain_active: &'a Chain) -> Self {
        Wallet {
            nWitnessCacheSize: 0,
            map_wallet: HashMap::new(),
            n_time_first_key: 0,

            chain_active,
            pcoins_tip,
            key_store: KeyStore::new(),
            seed: [0u8; 32],
        }
    }

    pub fn save_to_file(&self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::create(file_name)?;
        file.write_all(&self.seed)?;
        Ok(())
    }

    pub fn load_from_file(&mut self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let file_bytes = contents.as_bytes();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&file_bytes[0..32]);
        self.set_seed(seed);
        Ok(())
    }

    pub fn set_seed(&mut self, seed: [u8; 32]) -> SaplingPaymentAddress {
        self.seed = seed;
        let xsk = SaplingExtendedSpendingKey::master(&seed);
        self.add_spending_key_to_wallet(&xsk)
    }

    // Generate a new seed for the wallet, it will overwrite existing seed.
    pub fn generate_new_seed(&mut self) {
        let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
        self.set_seed(random_bytes);
    }

    //TODO, omit something for GUI
    pub fn scan_for_wallet_transactions(
        &mut self,
        pindex_start: Option<BlockIndex>,
        f_update: bool,
    ) {
        let mut ret = 0;
        let mut pindex = pindex_start;

        let mut my_tx_hashes = Vec::new();
        //while !pindex.is_none() && self.n_time_first_key > 0
        //TODO
        //&& pindex.unwrap().get_block_time() < self.n_time_first_key - 7200
        {
            //pindex = chain_active.next(pindex);
        }
        ShowProgress("Rescanning...", 0);

        while !pindex.is_none() {
            let block = {
                let t_pindex = pindex.clone();
                t_pindex.and_then(|p| read_block_from_disk(&p))
            };

            let block = {
                block.and_then(|b| {
                    for tx in b.vtx.iter() {
                        if self.add_to_wallet_if_invloving_me(tx, &b, f_update) {
                            my_tx_hashes.push(tx.hash);
                            ret += 1;
                        }
                    }
                    Some(b)
                })
            };

            let mut sapling_tree = {
                let t_pindex = pindex.clone();
                t_pindex.and_then(|p| {
                    p.pprev.and_then(|pp| {
                        self.pcoins_tip
                            .get_sapling_anchor_at(pp.hash_final_sapling_root)
                    })
                })
            };

            {
                let t_pindex = pindex.clone();
                self.chain_tip(
                    &t_pindex.unwrap(),
                    &block.unwrap(),
                    &mut sapling_tree.unwrap(),
                    true,
                );
            }

            pindex = pindex.and_then(|i| self.chain_active.next(i));
        }
    }

    pub fn find_my_sapling_notes(
        &self,
        tx: &Transaction,
    ) -> (NoteDataMap, SaplingIncomingViewingKeyMap) {
        let hash = tx.hash;

        let note_data = NoteDataMap::new();
        let viewing_keys_to_add = SaplingIncomingViewingKeyMap::new();

        for output in tx.v_shielded_output.iter() {
            let map_full_viewing_keys = self.key_store.get_map_full_viewing_keys();
            for (ivk, val) in map_full_viewing_keys.iter() {}
        }

        (note_data, viewing_keys_to_add)
    }

    fn add_sapling_incoming_view_key(
        &mut self,
        ivk: SaplingIncomingViewingKey,
        addr: SaplingPaymentAddress,
    ) -> bool {
        self.key_store.add_incoming_viewing_key(ivk, addr)
    }

    //TODO
    fn add_to_transparent_spends(&mut self, outpoint: &SaplingOutPoint, wtxid: FrHash) {}

    //TODO
    fn add_to_sapling_spends(&mut self, nullifier: U256, wtxid: &FrHash) {}

    fn add_to_spends(&mut self, wtxid: FrHash) {
        assert!(self.map_wallet.contains_key(&wtxid));

        let this_tx = self.map_wallet.get(&wtxid).unwrap();
        if this_tx.tx.is_coin_base() {
            return;
        }

        //TODO(can be deal later), uncomment it then compile error
        /*
        for txin in this_tx.tx.vin.iter() {
            self.add_to_transparent_spends(&txin.prevout, wtxid);
        }

        for spend in this_tx.tx.v_shielded_spend.iter() {
            self.add_to_sapling_spends(U256::from(spend.nullifier), &wtxid);
        }*/
    }

    fn add_to_wallet(&mut self, wtx_in: WalletTransaction, f_from_load_wallet: bool) -> bool {
        let hash = wtx_in.tx.hash;
        if f_from_load_wallet {
            //No DB yet
        } else {
            let mut f_inserted_new = false;
            if !self.map_wallet.contains_key(&hash) {
                self.map_wallet.insert(hash, wtx_in);
                f_inserted_new = true;
            }
            let mut wtx = self.map_wallet.get_mut(&hash).unwrap();

            //TODO, uncomment it then compile error
            //wtx.bind_wallet(&self);

            //TODO, uncomment it then compile error
            //self.update_sapling_nullifier_note_map_with_tx(wtx);

            if f_inserted_new {
                self.add_to_spends(hash);
            }
        }
        true
    }

    //TODO
    fn is_mine(&self, tx: &Transaction) -> bool {
        false
    }

    //TODO
    fn is_from_me(&self, tx: &Transaction) -> bool {
        false
    }

    pub fn add_to_wallet_if_invloving_me(
        &mut self,
        tx: &Transaction,
        block: &Block,
        f_update: bool,
    ) -> bool {
        let f_existed = self.map_wallet.contains_key(&tx.hash);
        if f_existed && !f_update {
            return false;
        }

        let (sapling_note_data, addresses_to_add) = self.find_my_sapling_notes(tx);
        for address_to_add in addresses_to_add {
            if !self.add_sapling_incoming_view_key(address_to_add.1, address_to_add.0) {
                return false;
            }
        }

        if f_existed || self.is_mine(tx) || self.is_from_me(tx) || sapling_note_data.len() > 0 {
            let mut wtx = WalletTransaction::new((*tx).clone());
            if sapling_note_data.len() > 0 {
                wtx.mapSaplingData = sapling_note_data;
            }
            return self.add_to_wallet(wtx, false);
        }
        true
    }

    pub fn get_sapling_note_witnesses(
        &self,
        notes: Vec<&SaplingOutPoint>,
    ) -> (Vec<Option<&SaplingWitness>>, Option<FrHash>) {
        let mut rt: Option<FrHash> = None;

        let mut witnesses = notes
            .iter()
            .map(|note| {
                self.map_wallet.get(&note.hash).and_then(|tx| {
                    tx.mapSaplingData.get(&note).and_then(|data| {
                        data.witnesses.front().and_then(|witness| {
                            let r = witness.root().unwrap();

                            match rt.clone() {
                                None => {
                                    rt = Some(r);
                                }
                                Some(root) => {
                                    assert_eq!(root, r);
                                }
                            }
                            Some(witness)
                        })
                    })
                })
            })
            .collect::<Vec<_>>();

        (witnesses, rt)
    }

    pub fn chain_tip(
        &mut self,
        pindex: &BlockIndex,
        pblock: &Block,
        saplingTree: &mut SaplingMerkleTree,
        added: bool,
    ) {
        if added {
            self.increment_note_witnesses(pindex, pblock, saplingTree);
        } else {
            self.decrement_note_witnesses(pindex);
        }
        self.update_sapling_nullifier_note_map_for_block(pblock);
    }

    fn update_sapling_nullifier_note_map_for_block(&mut self, pblock: &Block) {
        for tx in pblock.vtx.iter() {
            let hash = &tx.hash;
            let tx_is_ours = self.map_wallet.contains_key(hash);
            if tx_is_ours {
                let wtx = self.map_wallet.get_mut(hash);
                //self.update_sapling_nullifier_note_map_with_tx(&mut wtx.unwrap());
            }
        }
    }
    //            uint64_t position = nd.witnesses.front().position();
    //            SaplingFullViewingKey fvk = mapSaplingFullViewingKeys.at(nd.ivk);
    //            OutputDescription output = wtx.vShieldedOutput[op.n];
    //            auto optPlaintext = SaplingNotePlaintext::decrypt(output.encCiphertext, nd.ivk, output.ephemeralKey, output.cm);
    //            if (!optPlaintext) {
    //                // An item in mapSaplingNoteData must have already been successfully decrypted,
    //                // otherwise the item would not exist in the first place.
    //                assert(false);
    //            }
    //            auto optNote = optPlaintext.get().note(nd.ivk);
    //            if (!optNote) {
    //                assert(false);
    //            }
    //            auto optNullifier = optNote.get().nullifier(fvk, position);
    //            if (!optNullifier) {
    //                // This should not happen.  If it does, maybe the position has been corrupted or miscalculated?
    //                assert(false);
    //            }
    //            uint256 nullifier = optNullifier.get();
    //            mapSaplingNullifiersToNotes[nullifier] = op;
    //            item.second.nullifier = nullifier;

    //TODO
    fn update_sapling_nullifier_note_map_with_tx(&mut self, wtx: &mut WalletTransaction) {
        for (op, nd) in wtx.mapSaplingData.iter() {
            if nd.witnesses.is_empty() {

            } else {

            }
        }
    }

    ///**
    // * Update mapSaplingNullifiersToNotes, computing the nullifier from a cached witness if necessary.
    // */
    //void CWallet::UpdateSaplingNullifierNoteMapWithTx(CWalletTx& wtx) {
    //    LOCK(cs_wallet);
    //
    //    for (mapSaplingNoteData_t::value_type &item : wtx.mapSaplingNoteData) {
    //        SaplingOutPoint op = item.first;
    //        SaplingNoteData nd = item.second;
    //
    //        if (nd.witnesses.empty()) {
    //            // If there are no witnesses, erase the nullifier and associated mapping.
    //            if (item.second.nullifier) {
    //                mapSaplingNullifiersToNotes.erase(item.second.nullifier.get());
    //            }
    //            item.second.nullifier = boost::none;
    //        }
    //        else {
    //            uint64_t position = nd.witnesses.front().position();
    //            SaplingFullViewingKey fvk = mapSaplingFullViewingKeys.at(nd.ivk);
    //            OutputDescription output = wtx.vShieldedOutput[op.n];
    //            auto optPlaintext = SaplingNotePlaintext::decrypt(output.encCiphertext, nd.ivk, output.ephemeralKey, output.cm);
    //            if (!optPlaintext) {
    //                // An item in mapSaplingNoteData must have already been successfully decrypted,
    //                // otherwise the item would not exist in the first place.
    //                assert(false);
    //            }
    //            auto optNote = optPlaintext.get().note(nd.ivk);
    //            if (!optNote) {
    //                assert(false);
    //            }
    //            auto optNullifier = optNote.get().nullifier(fvk, position);
    //            if (!optNullifier) {
    //                // This should not happen.  If it does, maybe the position has been corrupted or miscalculated?
    //                assert(false);
    //            }
    //            uint256 nullifier = optNullifier.get();
    //            mapSaplingNullifiersToNotes[nullifier] = op;
    //            item.second.nullifier = nullifier;
    //        }
    //    }
    //}

    // void CWallet::UpdateSaplingNullifierNoteMapForBlock(const CBlock *pblock) {
    //    LOCK(cs_wallet);
    //
    //    for (const CTransaction& tx : pblock->vtx) {
    //        auto hash = tx.GetHash();
    //        bool txIsOurs = mapWallet.count(hash);
    //        if (txIsOurs) {
    //            UpdateSaplingNullifierNoteMapWithTx(mapWallet[hash]);
    //        }
    //    }
    //}

    pub fn decrement_note_witnesses(&mut self, pindex: &BlockIndex) {}

    //void CWallet::IncrementNoteWitnesses(const CBlockIndex* pindex,
    //                                     const CBlock* pblockIn,
    //                                     SproutMerkleTree& sproutTree,
    //                                     SaplingMerkleTree& saplingTree)
    //{
    pub fn increment_note_witnesses(
        &mut self,
        pindex: &BlockIndex,
        pblockIn: &Block,
        saplingTree: &mut SaplingMerkleTree,
    ) {
        for (_, wtx) in self.map_wallet.iter_mut() {
            copy_previous_witnesses(
                &mut wtx.mapSaplingData,
                pindex.nHeight,
                self.nWitnessCacheSize,
            );
        }
        if self.nWitnessCacheSize < WITNESS_CACHE_SIZE {
            self.nWitnessCacheSize += 1;
        }

        for tx in pblockIn.vtx.iter() {
            let hash = &tx.hash;
            let tx_is_ours = self.map_wallet.contains_key(hash);
            for (i, item) in tx.v_shielded_output.iter().enumerate() {
                //let repr = item.cmu.into_repr().as_ref();
                //let note_commitement = U256::from(repr);
                let cm = item.cmu;
                let note_commitement = FrHash(cm);
                let note_commitement_1 = note_commitement.clone();
                let note_commitement_2 = note_commitement.clone();
                saplingTree.append(note_commitement);

                for (_, wtx) in self.map_wallet.iter_mut() {
                    let cm = note_commitement_1.clone();
                    append_note_commitement(
                        &mut wtx.mapSaplingData,
                        pindex.nHeight,
                        self.nWitnessCacheSize,
                        cm,
                    );
                }

                if tx_is_ours {
                    let t_hash = tx.hash.clone();
                    let out_point = SaplingOutPoint {
                        hash: t_hash,
                        n: i as usize,
                    };
                    let nd = self.map_wallet.get_mut(&hash).unwrap();

                    witness_note_if_mine(
                        &mut nd.mapSaplingData,
                        pindex.nHeight,
                        self.nWitnessCacheSize,
                        note_commitement_2,
                        out_point,
                        saplingTree.witness().unwrap(),
                    );
                }
            }
        }

        for (_, wtx) in self.map_wallet.iter_mut() {
            update_witness_heights(
                &mut wtx.mapSaplingData,
                pindex.nHeight,
                self.nWitnessCacheSize,
            );
        }
    }

    pub fn get_new_z_address(&mut self) -> SaplingPaymentAddress {
        // TODO(xin): Change to rand.
        let seed = [0u8; 32];
        let xsk = SaplingExtendedSpendingKey::master(&seed);
        self.add_spending_key_to_wallet(&xsk)
    }

    pub fn add_z_key(
        &mut self,
        xsk: &SaplingExtendedSpendingKey,
        address: &SaplingPaymentAddress,
    ) -> bool {
        self.key_store
            .add_spending_key(xsk.clone(), address.clone())
    }

    pub fn add_spending_key_to_wallet(
        &mut self,
        xsk: &SaplingExtendedSpendingKey,
    ) -> SaplingPaymentAddress {
        let xfvk = SaplingExtendedFullViewingKey::from(xsk);
        let (_, address) = xfvk.default_address().unwrap();
        self.add_z_key(xsk, &address);
        address
    }

    //GetFilteredNotes(
    //    std::vector<SaplingNoteEntry>& saplingEntries,
    //    std::string address,
    //    int minDepth,
    //    bool ignoreSpent,
    //    bool requireSpendingKey)
    // support z_listunspent
    //TODO hu yuan
    pub fn get_filtered_notes() {}

    //bool CWallet::CreateTransaction(const vector<CRecipient>& vecSend, CWalletTx& wtxNew, CReserveKey& reservekey, CAmount& nFeeRet,
    //support sendmany
    pub fn create_transaction() {}

    //bool CWallet::CommitTransaction(CWalletTx& wtxNew, CReserveKey& reservekey)
    //suport sendmany
    pub fn commit_transaction(wtx_new: WalletTransaction) {}
}

fn copy_previous_witnesses(
    noteDataMap: &mut NoteDataMap,
    indexHeight: i32,
    nWitnessCacheSize: usize,
) {
    for (op, nd) in noteDataMap.iter_mut() {
        if nd.witnessHeight < indexHeight {
            assert!(nWitnessCacheSize >= nd.witnesses.len(), true);
            assert!(nd.witnessHeight == -1 || nd.witnessHeight == indexHeight - 1);
            if nd.witnesses.len() > 0 {
                nd.push_front(nd.front().unwrap())
            }
            if nd.witnesses.len() > WITNESS_CACHE_SIZE {
                nd.pop_back();
            }
        }
    }
}

fn append_note_commitement(
    noteDataMap: &mut NoteDataMap,
    indexHeight: i32,
    nWitnessCacheSize: usize,
    note_commitement: FrHash,
) {
    for (_, nd) in noteDataMap.iter_mut() {
        if nd.witnessHeight < indexHeight && nd.witnesses.len() > 0 {
            assert!(nWitnessCacheSize >= nd.witnesses.len());
            //nd.witnesses.front().
            //    and_then(|witness| witness.append(note_commitement));
            let cm = note_commitement.clone();
            nd.witnesses.front().unwrap().append(cm);
        }
    }
}

fn witness_note_if_mine(
    noteDataMap: &mut NoteDataMap,
    indexHeight: i32,
    nWitnessCacheSize: usize,
    note_commitement: FrHash,
    key: SaplingOutPoint,
    witness: SaplingWitness,
) {
    if noteDataMap.contains_key(&key) && noteDataMap[&key].witnessHeight < indexHeight {
        let nd = noteDataMap.get_mut(&key).unwrap();
        if nd.witnesses.len() > 0 {
            info!("Inconsistent witness cache state found");
            nd.witnesses.clear();
        }
        nd.push_front(witness);
        nd.witnessHeight = indexHeight - 1;
        assert!(nWitnessCacheSize >= nd.witnesses.len());
    }
}

fn update_witness_heights(
    noteDataMap: &mut NoteDataMap,
    indexHeight: i32,
    nWitnessCacheSize: usize,
) {
    for (op, nd) in noteDataMap.iter_mut() {
        if nd.witnessHeight < indexHeight {
            nd.witnessHeight = indexHeight;
            assert!(nWitnessCacheSize >= nd.witnesses.len());
        }
    }
}

//TODO
fn ShowProgress(title: &str, n: i32) {}

pub fn show() {
    println!("Wallet show");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_save_load_wallet() {
        let chain_active = Chain::new();
        let mut pcoins_tip = CoinViewCache::new();
        let mut wallet = Wallet::new(&mut pcoins_tip, &chain_active);
        wallet.set_seed([1u8; 32]);
        let addresses1 = wallet.key_store.get_sapling_payment_addresses();
        assert_eq!(addresses1.len(), 1);

        assert!(wallet.save_to_file("wallet.txt").is_ok());
        let mut pcoins_tip2 = CoinViewCache::new();
        let mut wallet2 = Wallet::new(&mut pcoins_tip2, &chain_active);
        assert!(wallet2.load_from_file("wallet.txt").is_ok());
        // Test address is the same from loaded wallet.
        let addresses2 = wallet2.key_store.get_sapling_payment_addresses();
        assert_eq!(addresses1.len(), addresses2.len());
        for add in addresses1.iter() {
            assert!(addresses2.contains(&add));
        }
    }

}
