# Atomic swap between Bitcoin and PKT with HTLC scripts
This program implements atomic swaps between Bitcoin and PKT. For this it uses a Hashed Time-Locked Contract (HTLC) script, as described [here](https://github.com/bitcoin/bips/blob/master/bip-0199.mediawiki) and in the next paragraph.

# HTLC
A HTCL script can be used to trade something outside of a blockchain. Let's assume Victor wants to buy a secret word from Peggy. They don't trust each other. How can we avoid that Victor pays the money, but Peggy doesn't tell him the word, or Peggy tells him the word, but he doesn't pay her?

To solve this problem, first they exchange public keys and agree upon a timeout threshold. Then Peggy provides a hash of the word. Now both can construct the same HTLC script and a P2SH address (pay-to-script) for it.

The script allows Peggy to spend the fund before the timeout, if she reveals the word. It also allows Victor to recover the fund after the timeout, in case Peggy doesn't do anything. It is important to note that the only allowed destination before the timeout is the public address of Peggy which they initially exchanged, and the public address of Victor after the timeout.

Next Victor sends funds to the P2SH address. Now Peggy can spend the funds, which reveals the word for Victor, or Victor can recover the fund after the timeout.

# Atomic swap
The atomic swap between Bitcoin and PKT is implemented with 2 HTLC scripts.

Alice wants to sell 1 BTC to Bob, and Bob pays Alice 100 PKT for it. First they exchange public keys:

- btc-alice-addr: Bitcoin address of Alice
- pkt-alice-addr: PKT address of Alice
- btc-bob-addr: Bitcoin address of Bob
- pkt-bob-addr: PKT address of Bob

Then Alice creates a secret word, and sends Bob the hash of this word. They can now both construct 2 HTLC scripts and P2SH addresses for it:

- btc-htlc-addr: P2SH address with HTLC script, which allows Bob to spend the 1 BTC to his btc-bob-addr, with the right secret word, or Alice to recover the funds to her btc-alice-addr after the timeout.
- pkt-htlc-addr: P2SH address with HTLC script, which allows Alice to spend the 100 PKT her pkt-alice-addr, with the right secret word, or Bob to recover the funds to his pkt-bob-addr after the timeout.

This nice symmetric interlocked construction implements 2 HTLC trades: the offchain BTC for PKT trade on the PKT blockchain, and the offchain PKT for BTC trade on the BTC blockchain, each with different seller and recover addresses, and locked by the same secret word.

Next Alice sends 1 BTC to btc-htlc-addr and Bob sends 100 PKT to pkt-htlc-addr. Then either of this can happen:

- Successful trade: Alice sends the 100 PKT of pkt-htlc-addr to her pkt-alice-addr. By doing this, she has to reveal the secret word, which then Bob uses to send the 1 BTC of btc-htlc-addr to his btc-bob-addr.
- Trade canceled: Bob recovers the 100 PKT from pkt-htlc-addr after the timeout, and Alice can recover her 1 BTC from btc-htlc-addr after the timeout. They can do this both independently, if one person doesn't fund a HTLC address, or if Alice doesn't reveal the secret word by sending the 100 PKT to her pkt-alice-addr.

# Implementation
The program implements the following commands:

- hash secret-word

  Hashes the secret word and returns the hash code in hex for it

- createhtlc seller-address buyer-address hash timeout

  Creates an address whose funds can be spent to the seller address with the secret word, or refunded to the buyer address after the timeout. Returns the address and the script.

# Bitcoin config file
For Linux, the config file is `~/.bitcoin/bitcoin.conf`. Example for binding the RPC port to the local IP address, which makes it accessible only from programs on the same computer:
```
txindex=1
[test]
server=1
rpcuser=bitcoinrpc
rpcpassword=test
rpcbind=127.0.0.1
rpcallowip=127.0.0.1
rpcport=9345
```

# Bitcoin testnet
For this project, we use the Bitcoin testnet, which you can start and sync like this with the standard Bitcoin client installation:
```
./bitcoind -testnet
```
The testnet blockchain needs about 30 GB on your harddisk, and a few hours to load initially.

For the tests, you need a wallet 
./bitcoin-cli -testnet createwallet "testwallet"

# testing the RPC interface

curl --user bitcoinrpc:test --data-binary '{"jsonrpc":"1.0","id":"curltext","method":"getblockchaininfo","params":[]}' -H 'content-type:text/plain;' http://127.0.0.1:9345

# testing

## prerequisite

The tests work on Linux. To run the tests, a bitcoin client must be in the path: the programs `bitcoind` and `bitcoin-cli`. Any running bitcoind command line program, or bitcoin Qt client, must be stopped before starting the tests.

## running the tests

The script `test.sh` tests the whole functionality of the software with the regtest network of the Bitcoin node, which is a local network on the PC where it is started. It creates the directory `bitcoin-data`, creates a wallet, mines some test Bitcoins, and then runs the Rust program to test the functionality.

# credits

Some source code is based on [this project](https://github.com/bitcoin-teleport/teleport-transactions). Original license file: [LICENSE-MIT](LICENSE-MIT).
