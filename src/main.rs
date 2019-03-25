mod block_chain;
mod coins;
mod incremental_tree;
mod key;
mod main_impl;
mod my;
mod other;
mod sendmany;
mod transaction;
mod transaction_builder;
mod wallet;

#[macro_use]
extern crate log;

use std::io;
use std::sync::mpsc;
use std::thread;
//use std::vec::Vec;

use crate::sendmany::SendMany;

use crate::coins::CoinViewCache;
use crate::key::key_store::KeyStore;
use crate::other::sanity_check::SanityChecker;
use crate::wallet::Wallet;
use crate::block_chain::{
    ChainActive,
};

fn main() {
    sendmany::show();
    wallet::show();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let chain_active = ChainActive::new();
        let mut pcoins_tip = CoinViewCache::new();
        let wallet = Wallet::new(&mut pcoins_tip, &chain_active);
        //let address_management = AddressManagement::new();
        let sanity_checker = SanityChecker::new();
        let key_store = KeyStore::new();

        let sender = SendMany {
            main_wallet: &wallet,
            //address_management: address_management,
            sanity_checker: sanity_checker,
            key_store: key_store,
        };

        //Setup work queue
        for item in rx {
            println!("Received: {}", item);
            let s = item as String;
            let params = s
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            sender.pre_send_many(params);
        }
        println!("Work queue thread end");
    });

    while true {
        //Take user action(sendTransaction etc)
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{} bytes read", n);
                println!("{}", input);

                tx.send(input).unwrap();
            }
            Err(error) => println!("error: {}", error),
        }
    }

    println!("Start success");
}

#[cfg(test)]
mod test {}
