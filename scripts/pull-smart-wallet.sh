#!/usr/bin/env sh

cd $(dirname $0)/..

mkdir -p artifacts/deploy/

solana program dump GokivDYuQXPZCWRkwMhdH2h91KpDQXBEmpgBgs55bnpH \
    artifacts/deploy/smart_wallet.so --url devnet    
