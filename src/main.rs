use bitcoin::blockdata::opcodes;
use bitcoin::blockdata::script::Builder;
use bitcoin::hashes::sha256::Hash as SHA256;
use bitcoin::hashes::Hash;
use bitcoin::util::amount::Amount;
use bitcoin::{Address, OutPoint, Script, SigHashType, Transaction, TxIn, TxOut, Txid};
use bitcoin::consensus::encode::{serialize_hex, deserialize};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::{App, Arg, Command, Parser};
use hex;
use serde::de::IntoDeserializer;
use std::env;
use std::str::FromStr;

use serde;
use serde::{Deserialize, Serialize};
use serde_json;

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
            Command::new("transaction")
                .about("Create HTLC script and transaction")
                .arg(Arg::new("bitcoin-address").required(true))
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
        let hash_string=hex::encode(hashvalue);

        let result: PreimageResult = PreimageResult {
            preimage: preimage_value.into(),
            hash: format!("{}", hash_string),
        };
        let lines = serde_json::to_string(&result).unwrap();
        println!("{}", lines);
    } else if let Some(transaction) = matches.subcommand_matches("transaction") {
        let bitcoin_address = transaction.value_of("bitcoin-address").unwrap();
        let public_address = Address::from_str(bitcoin_address).unwrap();

        let preimage_hash = hex::decode(transaction.value_of("preimage-hash").unwrap()).unwrap();
//        let preimage_hash = 

        // create RPC connection to bitcoin node
        let rpc = Client::new(
            format!("{host}:{port}").as_str(),
            Auth::UserPass(username.to_string(), password.to_string()),
        )
        .unwrap();

        // get private key for public key
        let private_key = rpc.dump_private_key(&public_address).unwrap();
        //println!("private key: {}", private_key);

        // create transaction

        // create HTLC script
        let locktime = 10; //blocks


        OP_IF
        [HASHOP] <digest> 
        OP_EQUALVERIFY 
        OP_DUP 
        OP_HASH160 <seller pubkey hash>            
    OP_ELSE
        <num> [TIMEOUTOP] OP_DROP OP_DUP OP_HASH160 <buyer pubkey hash>
    OP_ENDIF
    OP_EQUALVERIFY
    OP_CHECKSIG

    // create HTLC script according to
    // https://github.com/bitcoin/bips/blob/master/bip-0199.mediawiki
        let redeemscript = Builder::new()
           .push_opcode(opcodes::all::OP_IF)
    .push_opcode(opcodes::all::OP_SHA256)
            .push_slice(&preimage_hash[..])
            .push_opcode(opcodes::all::OP_EQUALVERIFY)
            .push_opcode(opcodes::all::OP_DUP)
      //          .push_key(&pub_hashlock)
            .push_opcode(opcodes::all::OP_ELSE)
                .push_int(locktime)
                .push_opcode(opcodes::all::OP_CSV)
                .push_opcode(opcodes::all::OP_DROP)
        //        .push_key(&pub_timelock)
            .push_opcode(opcodes::all::OP_ENDIF)
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
