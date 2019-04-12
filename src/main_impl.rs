//Functions and Operation that related to chain operation

use crate::block_chain::{Block, BlockIndex, ValidationState};
use crate::coins::{CoinViewCache, CoinsView};
use crate::key::key_management::FrHash;
use crate::key::proof::ProofVerifier;
use crate::transaction::Transaction;
use crate::txmempool::{TxMemPool, TxMemPoolEntry};
use crate::wallet::Wallet;
use crate::zkp::{OUTPUT_VK, SPEND_VK};

use bellman::groth16::{Proof};
use ethereum_types::U256;
use pairing::bls12_381::Bls12;
use rustzcash::{};
use sapling_crypto::redjubjub::Signature;
use std::collections::hash_set::HashSet;
use zcash_primitives::JUBJUB;
use zcash_proofs::sapling::{SaplingVerificationContext};

//bool ReadBlockFromDisk(CBlock& block, const CBlockIndex* pindex)

pub fn read_block_from_disk(pindex: &BlockIndex) -> Option<Block> {
    None
}

/**
 * Make the best chain active, in multiple steps. The result is either failure
 * or an activated best chain. pblock is either NULL or a pointer to a block
 * that is already loaded (to avoid loading it again from disk).
 */
//bool ActivateBestChain(CValidationState &state, CBlock *pblock) {
pub fn active_best_chain() {}

/**
 * Try to make some progress towards making pindexMostWork the active block.
 * pblock is either NULL or a pointer to a CBlock corresponding to pindexMostWork.
 */
//static bool ActivateBestChainStep(CValidationState &state, CBlockIndex *pindexMostWork, CBlock *pblock) {
pub fn active_best_chain_step() {
    //call connect_tip
}

//bool static ConnectTip(CValidationState &state, CBlockIndex *pindexNew, CBlock *pblock)
pub fn connect_tip(
    pcoins_tip: &mut CoinViewCache,
    wallet: &mut Wallet,
    state: &ValidationState,
    pindex_new: &BlockIndex,
    pblock: &Block,
) {
    //call connect_block
    //call wallet.chain_tip

    //SproutMerkleTree oldSproutTree;
    //SaplingMerkleTree oldSaplingTree;
    //assert(pcoinsTip->GetSproutAnchorAt(pcoinsTip->GetBestAnchor(SPROUT), oldSproutTree));
    //assert(pcoinsTip->GetSaplingAnchorAt(pcoinsTip->GetBestAnchor(SAPLING), oldSaplingTree));
    //let old_sapling_tree
    //    = pcoins_tip.get_sapling_anchor_at(pcoins_tip.get_best_anchor());
    let old_sapling_tree = pcoins_tip
        .get_best_anchor()
        .and_then(|anchor| pcoins_tip.get_sapling_anchor_at(anchor));
    connect_block(pblock, state, pindex_new, pcoins_tip, false);

    wallet.chain_tip(pindex_new, pblock, &mut old_sapling_tree.unwrap(), true);
}

/**
 * Disconnect chainActive's tip. You probably want to call mempool.removeForReorg and
 * mempool.removeWithoutBranchId after this, with cs_main held.
 */
//bool static DisconnectTip(CValidationState &state, bool fBare = false) {
pub fn disconnect_tip() {}

//bool ConnectBlock(const CBlock& block, CValidationState& state,
// CBlockIndex* pindex, CCoinsViewCache& view, bool fJustCheck)

pub fn update_coins(tx: &Transaction, inputs: &mut CoinViewCache, height: i32) {
    /*if !tx.is_coin_base() {
        for txin in tx.vin {
        }
    }*/

    inputs.set_nullifiers(tx, true);

    //add outputs
}

pub fn connect_block(
    block: &Block,
    state: &ValidationState,
    pindex: &BlockIndex,
    view: &mut CoinViewCache,
    f_just_check: bool,
) {
    /*BOOST_FOREACH(const CTransaction& tx, block.vtx) {
        const CCoins* coins = view.AccessCoins(tx.GetHash());
        if (coins && !coins->IsPruned())
            return state.DoS(100, error("ConnectBlock(): tried to overwrite transaction"),
                             REJECT_INVALID, "bad-txns-BIP30");
    }*/
    let sapling_tree = view
        .get_best_anchor()
        .and_then(|anchor| view.get_sapling_anchor_at(anchor))
        .unwrap();

    for tx in block.vtx.iter() {
        update_coins(tx, view, pindex.nHeight);
        for output in tx.v_shielded_output.iter() {
            sapling_tree.append(FrHash(output.cmu));
        }
    }

    view.push_anchor(sapling_tree);

    view.set_best_block(pindex.get_block_hash());
}

pub fn check_block(
    block: &Block,
    state: &ValidationState,
    verifier: &ProofVerifier,
    f_check_POW: bool,
    f_check_merkle_root: bool,
) -> bool {
    if !check_block_header(block, state, f_check_POW) {
        return false;
    }

    true
}

pub fn check_block_header(block: &Block, state: &ValidationState, f_check_POW: bool) -> bool {
    true
}

//bool ProcessNewBlock(CValidationState &state, CNode* pfrom,
// CBlock* pblock, bool fForceProcessing, CDiskBlockPos *dbp)

pub fn process_new_block() {
    accept_block();
    //call active_best_chain
}

//bool AcceptBlock(CBlock& block, CValidationState& state,
// CBlockIndex** ppindex, bool fRequested, CDiskBlockPos* dbp)

pub fn accept_block() {
    accept_block_header()
}

//bool AcceptBlockHeader(const CBlockHeader& block,
// CValidationState& state, CBlockIndex** ppindex)

pub fn accept_block_header() {}

pub fn check_transaction_without_proof_verification(
    tx: &Transaction,
    state: &ValidationState,
) -> bool {
    let mut v_sapling_nullifiers = HashSet::new();
    for spend in tx.v_shielded_spend.iter() {
        if v_sapling_nullifiers.contains(&spend.nullifier) {
            return false;
        }
        v_sapling_nullifiers.insert(spend.nullifier);
    }
    if tx.is_coin_base() {
        if tx.v_shielded_spend.len() > 0 {
            return false;
        }
        if tx.v_shielded_output.len() > 0 {
            return false;
        }
    } else {

    }
    true
}

pub fn check_transaction(tx: &Transaction, state: &ValidationState) -> bool {
    if !check_transaction_without_proof_verification(tx, state) {
        return false;
    }
    true
}

//TODO wu xin
//check out SignatureHash implements
fn signature_hash(tx: &Transaction) -> [u8; 32] {
    [0u8; 32]
}

// Check spend, output, and value balance signature.
pub fn contextual_check_transaction(tx: &Transaction, state: &ValidationState) -> bool {
    if !tx.v_shielded_spend.is_empty() || !tx.v_shielded_output.is_empty() {
        let mut ctx = SaplingVerificationContext::new();
        let sighash = signature_hash(&tx);

        for spend in &tx.v_shielded_spend {
            let zkproof = match Proof::<Bls12>::read(&spend.zkproof[..]) {
                Ok(p) => p,
                Err(_) => return false,
            };
            if !ctx.check_spend(
                spend.cv,
                spend.anchor,
                &spend.nullifier,
                spend.rk.clone(),
                &sighash,
                spend.spend_auth_sig.unwrap(),
                zkproof,
                &SPEND_VK,
                &JUBJUB)   {
                return false;
            }
        }

        for output in &tx.v_shielded_output {
            let zkproof = match Proof::<Bls12>::read(&output.zkproof[..]) {
                Ok(p) => p,
                Err(_) => return false,
            };
            if !ctx.check_output(
                output.cv,
                output.cmu,
                output.ephemeral_key,
                zkproof,
                &OUTPUT_VK,
                &JUBJUB)   {
                return false;
            }
        }
        return ctx.final_check(tx.balancing_value, &sighash, Signature::read(&tx.binding_sig[..]).unwrap(), &JUBJUB)
    }
    true
}

fn contextual_check_inputs() -> bool {
    true
}

//TODO
fn check_final_tx() -> bool {
    true
}

pub fn accept_to_mem_pool<'a>(
    pool: &'a mut TxMemPool<'a>,
    state: &ValidationState,
    tx: &'a Transaction,
    pcoins_tip: &mut CoinViewCache,
) -> bool {
    if !check_transaction(tx, state) {
        return false;
    }
    if !contextual_check_transaction(tx, state) {
        return false;
    }
    // Coinbase is only valid in a block, not as a loose transaction
    if tx.is_coin_base() {
        return false;
    }
    if !check_final_tx() {
        return false;
    }

    let hash = tx.hash;
    if pool.exists(hash) {
        return false;
    }

    for txin in tx.vin.iter() {
        if pool.mapNextTx.contains_key(&txin.prevout) {
            //Disable replacement feature for now
            return false;
        }
    }

    for spend_description in tx.v_shielded_spend.iter() {
        if pool.nullifier_exists(U256::from(spend_description.nullifier)) {
            return false;
        }
    }

    {
        //TODO, backed view
        let mut view = pcoins_tip;

        if view.have_coins(hash) {
            return false;
        }

        for txin in tx.vin.iter() {
            if !view.have_coins(txin.prevout.hash) {
                //pf_missing_inputs
                return false;
            }
        }

        if !view.have_inputs(tx) {
            return false;
        }

        if !view.have_shield_requirements(tx) {
            return false;
        }

        let entry = TxMemPoolEntry::new(tx);
        //let entry_ptr: &'a TxMemPoolEntry = &entry;

        if !contextual_check_inputs() {
            return false;
        }

        //Different check, with different flags
        if !contextual_check_inputs() {
            return false;
        }

        pool.add_unchecked(hash, entry);
    }

    true
}