#!/bin/bash

trap "exit" INT TERM
trap "kill 0" EXIT

function mine_blocks {
    while true; do
	printf "Mining 1 block to address: $MINING_ADDRESS ..."
	docker compose run btcctl generate 1
	sleep 1
    done
}


cd itests
docker compose down --volumes --remove-orphans
docker compose build
docker compose up -d

# Initialize the blockchain with miner rewards going to the test client.

client_address=""
while [ "$client_address" = "" ]
do
    new_address_output=$(docker exec -it test_lnd_client lncli --network=simnet newaddress np2wkh)
    echo "new_address_output:"
    echo $new_address_output
    client_address=$(echo $new_address_output | jq .address -r)
    sleep 2
done

export MINING_ADDRESS="$client_address docker compose up -d btcd"
echo "Mining 400 blocks to address: $client_address ..."
docker compose run btcctl generate 400
echo "Finished mining blocks."
sleep 30

echo "Continue mining blocks, 1 every 10 seconds."
mine_blocks &
echo "Background mining task is in background..."

echo "Running test.sh...."
docker compose run test ./test.sh

echo "Shutting down itest..."
# docker compose down
