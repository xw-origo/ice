use crate::key::key_management::{FrHash, SaplingOutputDescription, SaplingSpendDescription};
use crate::script::Script;
use crate::sendmany::CAmount;
use crate::sendmany::SaplingNoteData;
use crate::sendmany::SaplingOutPoint;
use ethereum_types::U256;
use std::collections::HashMap;

pub type NoteDataMap = HashMap<SaplingOutPoint, SaplingNoteData>;

//Program cache
pub struct WalletTransaction {
    //std::map<SaplingOutPoint, SaplingNoteData> mapSaplingData;
    pub mapSaplingData: NoteDataMap,
}

/** An inpoint - a combination of a transaction and an index n into its vin */
pub struct SaplingInPoint<'a> {
    pub ptx: &'a Transaction,
    pub n: usize,
}

impl<'a> SaplingInPoint<'a> {
    pub fn new(tx: &'a Transaction, index: usize) -> Self {
        SaplingInPoint { ptx: tx, n: index }
    }
}

pub struct TxIn {
    pub prevout: SaplingOutPoint,
    pub script_sig: Script,
}

pub struct TxOut {
    pub n_value: i64,
    pub script_pub_key: Script,
}

impl TxOut {
    pub fn is_null(&self) -> bool {
        self.n_value == -1
    }
}
//In DB and network
pub struct Transaction {
    pub hash: FrHash, //U256,

    pub vin: Vec<TxIn>,
    pub vout: Vec<TxOut>,
    pub v_shielded_spend: Vec<SaplingSpendDescription>,
    pub v_shielded_output: Vec<SaplingOutputDescription>,
    pub balancing_value: i64,
    pub binding_sig: [u8; 64],
}

impl Transaction {
    //TODO
    pub fn is_coin_base(&self) -> bool {
        false
    }
}

pub struct TxUndo {
    vpreout: Vec<TxInUndo>,
}

pub struct TxInUndo {}
