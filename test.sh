#!/bin/bash

# helper functions
rpc() {
    bitcoin-cli -rpcuser=bitcoinrpc -rpcpassword=test -rpcport=8333 -regtest -datadir=$BITCOIN_DATADIR "$@"
}

echo "shutdown any running bitcoin daemon"
killall bitcoind
sleep 5

# create new datadir
BITCOIN_DATADIR=`pwd`/bitcoin-data
rm -rf $BITCOIN_DATADIR
mkdir $BITCOIN_DATADIR

echo "starting new bitcoin daemon for regtest"
bitcoind -fallbackfee=0.0000001 -daemon -server -regtest -datadir=$BITCOIN_DATADIR -rpcbind=127.0.0.1 -rpcuser=bitcoinrpc -rpcpassword=test -rpcallowip=127.0.0.1 -rpcport=8333
sleep 5

echo "creating wallet"
rpc createwallet swap > /dev/null

echo "create Bitcoin address"
ADDR=`rpc getnewaddress`

echo "mine some Bitcoin to it. Needs at least 100 blocks before it is usable."
rpc generatetoaddress 150 $ADDR > /dev/null

echo "balance for address $ADDR: `rpc getbalance`"

ADDR2=`rpc getnewaddress`
echo "send 10 BTC to $ADDR2"
rpc sendtoaddress "$ADDR2" 10 > /dev/null

BAL2=`rpc listreceivedbyaddress 0 true | jq -r ".[] | select( .address == \"$ADDR2\" ) | .amount"`
echo "balance for address $ADDR2: $BAL2"

# ./target/debug/pktbtcswap $ADDR2

#echo "create Bitcoin address"
#ADDR=`rpc -named getnewaddress address_type=bech32`

