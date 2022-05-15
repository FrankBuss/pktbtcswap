use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::consensus::encode::serialize;
use bitcoin::consensus::encode::{deserialize, serialize_hex};
use bitcoin::hashes::hash160::Hash as Hash160;
use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::sha256::Hash as SHA256;
use bitcoin::hashes::Hash;
use bitcoin::util::address::Payload;
use bitcoin::util::amount::Amount;
use bitcoin::util::base58;
use bitcoin::PubkeyHash;
use bitcoin::{Address, OutPoint, PublicKey, Script, SigHashType, Transaction, TxIn, TxOut, Txid};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::{App, Arg, Command, Parser};
use hex;
use serde::de::IntoDeserializer;
use std::env;
use std::io::Read;
use std::str::FromStr;

use serde;
use serde::{Deserialize, Serialize};
use serde_json;

// TODO: use this when integrated in the next stable Rust compiler version
//#![feature(proc_macro_hygiene)]
//use bitcoin_script::bitcoin_script;

fn get_pubkey_hash(bitcoin_address_string: &str) -> PubkeyHash {
    let bitcoin_address = Address::from_str(bitcoin_address_string).unwrap();
    let payload = bitcoin_address.payload;
    if let Payload::PubkeyHash(hash) = payload {
        return hash;
    } else {
        panic!("{} is not a P2PKH address", bitcoin_address_string);
    }
}

fn main() {
    let mut app = Command::new("pktbtcswap")
        .version("0.1")
        .author("Frank Buss <fb@frank-buss.de>")
        .about("Atomic swap implementation between Bitcoin and PKT")
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .help("RPC host")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .help("RPC port")
                .default_value("8333"),
        )
        .arg(
            Arg::new("username")
                .short('u')
                .long("username")
                .help("RPC username")
                .default_value("bitcoinrpc"),
        )
        .arg(
            Arg::new("password")
                .short('a')
                .long("password")
                .help("RPC password")
                .default_value("test"),
        )
        .subcommand(
            Command::new("preimage")
                .about("Create a preimage and hashed preimage")
                .arg(Arg::new("preimage-value").required(true)),
        )
        .subcommand(
            Command::new("script")
                .about("Create HTLC script")
                .arg(Arg::new("bitcoin-to-address").required(true))
                .arg(Arg::new("preimage-hash").required(true)),
        )
        .subcommand(
            Command::new("transaction")
                .about("Create HTLC script and transaction")
                .arg(Arg::new("unspent-txid").required(true))
                .arg(Arg::new("unspent-vout").required(true))
                .arg(Arg::new("redeem-script").required(true))
                .arg(Arg::new("bitcoin-to-address").required(true))
                .arg(Arg::new("amount").required(true))
                .arg(Arg::new("preimage-hash").required(true)),
        );
    let matches = app.clone().get_matches();

    // create RPC connection to bitcoin node
    let host = matches.value_of("host").unwrap();
    let port: u32 = matches.value_of("port").unwrap().parse().unwrap();
    let username = matches.value_of("username").unwrap();
    let password = matches.value_of("password").unwrap();

    //    let interface = matches.value_of("interface").unwrap_or("");

    if let Some(preimage) = matches.subcommand_matches("preimage") {
        // create preimage and hash it, and print it as JSON
        let preimage_value = preimage.value_of("preimage-value").unwrap();

        let hashvalue = SHA256::hash(preimage_value.as_bytes());

        #[derive(Serialize, Debug)]
        struct PreimageResult {
            preimage: String,
            hash: String,
        }
        let hash_string = hex::encode(hashvalue);

        let result: PreimageResult = PreimageResult {
            preimage: preimage_value.into(),
            hash: format!("{}", hash_string),
        };
        let lines = serde_json::to_string(&result).unwrap();
        println!("{}", lines);
    } else if let Some(transaction) = matches.subcommand_matches("script") {
        //RAW_TX=`brpc createrawtransaction "[{\"txid\":\"$UNSPENT_TXID\",\"vout\":$UNSPENT_VOUT}]" "[{\"$ADDR3\":$AMOUNT}]" 0 true`
        //RAW_TX=`brpc createrawtransaction "[{\"txid\":\"$UNSPENT_TXID\",\"vout\":$UNSPENT_VOUT}]" "[{\"$ADDR3\":$AMOUNT}]" 0 true`

        // testing a normal transfer

        // get commandline parameters
        let bitcoin_to_address_string = transaction.value_of("bitcoin-to-address").unwrap();
        let preimage_hash_string = transaction.value_of("preimage-hash").unwrap();

        // parse bitcoin address, and get the hashed public key part
        let to_pubkey_hash = get_pubkey_hash(bitcoin_to_address_string);

        // create script
        let redeemscript: Script = Builder::new()
            .push_opcode(opcodes::all::OP_DUP)
            .push_opcode(opcodes::all::OP_HASH160)
            .push_slice(&to_pubkey_hash[..])
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();

        #[derive(Serialize, Debug)]
        struct ScriptResult {
            script: String,
            p2sh_address: String,
        }
        let network = Address::from_str(bitcoin_to_address_string)
            .unwrap()
            .network;
        let p2sh_address = Address::p2sh(&redeemscript, network);
        let result: ScriptResult = ScriptResult {
            script: hex::encode(&redeemscript.as_bytes()),
            p2sh_address: p2sh_address.to_string(),
        };
        let lines = serde_json::to_string(&result).unwrap();
        println!("{}", lines);
    } else if let Some(transaction) = matches.subcommand_matches("transaction") {
        //RAW_TX=`brpc createrawtransaction "[{\"txid\":\"$UNSPENT_TXID\",\"vout\":$UNSPENT_VOUT}]" "[{\"$ADDR3\":$AMOUNT}]" 0 true`
        //RAW_TX=`brpc createrawtransaction "[{\"txid\":\"$UNSPENT_TXID\",\"vout\":$UNSPENT_VOUT}]" "[{\"$ADDR3\":$AMOUNT}]" 0 true`

        // testing a normal transfer

        // get commandline parameters
        let unspent_txid_string = transaction.value_of("unspent-txid").unwrap();
        let unspent_vout_string = transaction.value_of("unspent-vout").unwrap();
        let redeem_script_string = transaction.value_of("redeem-script").unwrap();
        let redeem_script = Script::from_hex(redeem_script_string).unwrap();
        let bitcoin_to_address_string = transaction.value_of("bitcoin-to-address").unwrap();
        let bitcoin_to_address = Address::from_str(bitcoin_to_address_string).unwrap();
        let amount_string = transaction.value_of("amount").unwrap();
        let preimage_hash_string = transaction.value_of("preimage-hash").unwrap();

        // parse bitcoin address, and get the hashed public key part
        let to_pubkey_hash = get_pubkey_hash(bitcoin_to_address_string);

        // create script
        let scriptSig: Script = Builder::new()
            .push_slice(&redeem_script[..])
            .into_script();

        // create transaction
        let amount: f64 = amount_string.parse().unwrap();
        let tx = Transaction {
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_str(unspent_txid_string).unwrap(),
                    vout: unspent_vout_string.parse().unwrap(),
                },
                sequence: 0xfffffffd,
                witness: Vec::new(),
                script_sig: scriptSig,
            }],
            output: vec![TxOut {
                script_pubkey: bitcoin_to_address.script_pubkey(),
                value: Amount::from_btc(amount).unwrap().as_sat(),
            }],
            lock_time: 0,
            version: 2,
        };

        let data = serialize(&tx);

        #[derive(Serialize, Debug)]
        struct TransactionResult {
            transaction: String,
        }
        let result: TransactionResult = TransactionResult {
            transaction: hex::encode(data),
        };
        let lines = serde_json::to_string(&result).unwrap();
        println!("{}", lines);

        return;

        let bitcoin_address_string = transaction.value_of("bitcoin-address").unwrap();
        let bitcoin_address = Address::from_str(bitcoin_address_string).unwrap();
        println!("bitcoin = {:02x?}", bitcoin_address);
        let data = base58::from_check(bitcoin_address_string).unwrap();
        if data.len() != 21 {
            println!("invalid bitcoin address");
            return;
        }
        let bytes = &data[..];
        println!("hex = {:02x?}", bytes);

        let preimage_hash = hex::decode(transaction.value_of("preimage-hash").unwrap()).unwrap();
        //        let preimage_hash =

        // create RPC connection to bitcoin node
        let rpc = Client::new(
            format!("{host}:{port}").as_str(),
            Auth::UserPass(username.to_string(), password.to_string()),
        )
        .unwrap();

        // get private key for public key
        //let private_key = rpc.dump_private_key(&public_address).unwrap();
        //println!("private key: {}", private_key);

        // create transaction

        // create HTLC script

        // create HTLC script according to
        // https://github.com/bitcoin/bips/blob/master/bip-0199.mediawiki
        /*
        This is the script:
            OP_IF
                OP_SHA256 <digest>
                OP_EQUALVERIFY
                OP_DUP
                OP_HASH160 <seller pubkey hash>
            OP_ELSE
                <num> OP_CHECKLOCKTIMEVERIFY
                OP_DROP
                OP_DUP
                OP_HASH160 <buyer pubkey hash>
            OP_ENDIF
            OP_EQUALVERIFY
            OP_CHECKSIG
        With the nightly build of Rust, this crate could be used:
        https://docs.rs/bitcoin-script/latest/bitcoin_script/
        and it would simplify the syntax. But I will use the latest stable Rust version,
        and will update it once the hygienec macro features is integrated in Rust.
            */

        let locktime = 10; //blocks

        let preimage = "test";
        let preimage_hash = Hash160::hash(preimage.as_bytes());

        //            let seller_pubkey: Address = Address::from_str(bitcoin_address).unwrap();
        //            let seller_pubkey_hash = Hash160::hash(seller_pubkey.to_by);

        let redeemscript = Builder::new()
            .push_opcode(opcodes::all::OP_IF)
            .push_opcode(opcodes::all::OP_SHA256)
            .push_slice(&preimage_hash[..])
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_DUP)
            //            .push_slice(&seller_pubkey_hash[..])
            //.push_key(bitcoin_from_address.payload.)
            .push_opcode(opcodes::all::OP_HASH160)
            .push_opcode(opcodes::all::OP_ELSE)
            .push_int(locktime)
            .push_opcode(opcodes::all::OP_CLTV) /* OP_CHECKLOCKTIMEVERIFY */
            .push_opcode(opcodes::all::OP_DROP)
            .push_opcode(opcodes::all::OP_DUP)
            //                .push_slice(&buyer_pubkey_hash[..])
            .push_opcode(opcodes::all::OP_HASH160)
            .push_opcode(opcodes::all::OP_ENDIF)
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_CHECKSIG)
            .into_script();
        println!("redeemscript hex = {:x}", redeemscript);
        println!("redeemscript len = {}", redeemscript.len());
    } else {
        app.print_help().unwrap();
        println!("");
        return;
    }
}
