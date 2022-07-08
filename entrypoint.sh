#!/bin/bash

# Required environment variables
# export SQUEAKROAD_DB_URL=
# export SQUEAKROAD_ADMIN_USERNAME=
# export SQUEAKROAD_ADMIN_PASSWORD=
# export SQUEAKROAD_LND_HOST=
# export SQUEAKROAD_LND_PORT=
# export SQUEAKROAD_LND_TLS_CERT_PATH=
# export SQUEAKROAD_LND_MACAROON_PATH=

# Generate a secret
echo $(openssl rand -base64 32)
export ROCKET_SECRET_KEY=$(openssl rand -base64 32)

# Configure the address and port
export ROCKET_ADDRESS=0.0.0.0
export ROCKET_PORT=8000

# if lnd enabled, attempt to connect
exec squeakroad
