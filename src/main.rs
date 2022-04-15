use bitcoin::util::amount::Amount;
use bitcoin::{Address, OutPoint, Script, SigHashType, Transaction, TxIn, TxOut, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::env;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = env::args().collect();
    let public_address_string = &args[1];
    let public_address = Address::from_str(public_address_string.as_str()).unwrap();

    // create RPC connection to bitcoin node
    let host = "127.0.0.1";
    let port = 8333;
    let rpc_username = "bitcoinrpc";
    let rpc_password = "test";
    let rpc = Client::new(
        format!("{host}:{port}").as_str(),
        Auth::UserPass(rpc_username.to_string(), rpc_password.to_string()),
    )
    .unwrap();

    // test call
    //let best_block_hash = rpc.get_best_block_hash().unwrap();
    //println!("best block hash: {}", best_block_hash);

    // get private key for public key
    let private_key = rpc.dump_private_key(&public_address).unwrap();
    println!("private key: {}", private_key);
}
