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

Some source code is based on [this project](https://github.com/bitcoin-teleport). Original license file: [LICENSE-MIT](LICENSE-MIT).
