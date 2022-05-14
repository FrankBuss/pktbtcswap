#!/bin/bash

# helper functions
brpc() {
    bitcoin-cli -rpcuser=bitcoinrpc -rpcpassword=test -rpcport=8333 -regtest -datadir=$BITCOIN_DATADIR "$@"
}
prpc() {
    pktctl --rpcuser=pktrpc --rpcpass=test --rpcserver=127.0.0.1:8334 --regtest --datadir=$PKT_DATADIR "$@"
}
bitcoin_received() {
    brpc listreceivedbyaddress 0 true | jq -r ".[] | select( .address == \"$1\" ) | .amount"
}
bitcoin_unspent() {
    brpc listunspent | jq -r ".[] | select( .address == \"$2\" ) | .$1"
}
bitcoin_mine() {
    brpc generatetoaddress $1 $ADDR1 > /dev/null
}

echo "shutdown any running bitcoin and pkt daemon"
killall bitcoind
killall pktd
sleep 5

# create new Bitcoin datadir
BITCOIN_DATADIR=`pwd`/bitcoin-data
rm -rf $BITCOIN_DATADIR
mkdir $BITCOIN_DATADIR

echo "starting new bitcoin daemon for regtest"
bitcoind -fallbackfee=0.0000001 -daemon -server -regtest -datadir=$BITCOIN_DATADIR -rpcbind=127.0.0.1 -rpcuser=bitcoinrpc -rpcpassword=test -rpcallowip=127.0.0.1 -rpcport=8333
sleep 5

echo "creating wallet"
brpc createwallet swap > /dev/null

echo "create Bitcoin address"
ADDR1=`brpc getnewaddress addr1 legacy`

echo "mine some Bitcoin to it. Needs at least 100 blocks before it is usable."
bitcoin_mine 150

echo "balance for address $ADDR1: `brpc getbalance`"

ADDR2=`brpc getnewaddress addr2 legacy`
echo "send 10 BTC to $ADDR2"
brpc sendtoaddress "$ADDR2" 10 > /dev/null

# confirm transaction
bitcoin_mine 1

# show balance
BAL2=`bitcoin_received $ADDR2`
echo "balance for address $ADDR2: $BAL2"

ADDR3=`brpc getnewaddress addr3 legacy`

UNSPENT_TXID=`bitcoin_unspent txid $ADDR2`
UNSPENT_VOUT=`bitcoin_unspent vout $ADDR2`
UNSPENT_AMOUNT=`bitcoin_unspent amount $ADDR2`
UNSPENT_SCRIPT_PUB_KEY=`bitcoin_unspent scriptPubKey $ADDR2`
AMOUNT="9.99999"
RAW_TX=`brpc createrawtransaction "[{\"txid\":\"$UNSPENT_TXID\",\"vout\":$UNSPENT_VOUT}]" "[{\"$ADDR3\":$AMOUNT}]" 0 true`
brpc decoderawtransaction $RAW_TX

SIGNED_TX=`brpc signrawtransactionwithwallet $RAW_TX "[{\"txid\":\"$TXID\",\"vout\":$VOUT,\"scriptPubKey\":\"$SCRIPT_PUB_KEY\",\"amount\":$UNSPENT_AMOUNT}]" | jq -r ".hex"`

#brpc sendrawtransaction $SIGNED_TX

# confirm transaction
bitcoin_mine 1

BAL3=`bitcoin_balance $ADDR3`
echo "balance for address $ADDR3: $BAL3"
