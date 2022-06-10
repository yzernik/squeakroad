#!/bin/bash


while true; do
    client_address=$(docker exec -it test_lnd_client lncli --network=simnet newaddress np2wkh | jq .address -r)
    MINING_ADDRESS=$client_address docker compose up -d btcd
    echo "Mining 1 blocks to address: $client_address ..."
    docker compose run btcctl generate 400
    sleep 10
done
