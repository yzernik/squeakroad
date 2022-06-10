# configuration


#### configs


config | type | valid values | has default | default value | environment variable | description
--- | --- | --- | --- | --- | --- | ---
node.network | string | ['mainnet', 'testnet', 'regtest', 'simnet'] | yes | "testnet" | SQUEAKNODE_NODE_NETWORK | Which network to use.
node.price_msat | int | [0,...] | yes | 10000 | SQUEAKNODE_NODE_PRICE_MSAT | The price to sell squeaks to other peers in millisatoshis.
node.max_squeaks | int | [0,...] | yes | 10000 | SQUEAKNODE_NODE_MAX_SQUEAKS | The absolute maximum number of squeaks allowed in the database.
node.max_squeaks_per_public_key_per_block | int | [0,...] | yes | 1000 | SQUEAKNODE_NODE_MAX_SQUEAKS_PER_PUBLIC_KEY_PER_BLOCK | The maximum number of squeaks for an any public key with any block height.
node.sqk_dir_path | string | | yes | "<USER_HOME>/.sqk" | SQUEAKNODE_NODE_SQK_DIR_PATH | The directory to store application data (only if using sqlite as database backend).
node.log_level | string | | yes | "INFO" | SQUEAKNODE_NODE_LOG_LEVEL | The log level to use.
node.sent_offer_retention_s | int | [0,...] | yes | 86400 | SQUEAKNODE_NODE_SENT_OFFER_RETENTION_S | The amount of time in seconds to keep a sent offer after expiry before deleting it.
node.received_offer_retention_s | int | [0,...] | yes | 86400 | SQUEAKNODE_NODE_RECEIVED_OFFER_RETENTION_S | The amount of time in seconds to keep a received offer after download before deleting it.
node.subscribe_invoices_retry_s | int | [0,...] | yes | 10 | SQUEAKNODE_NODE_SUBSCRIBE_INVOICES_RETRY_S | The amount of time in seconds to wait after a subscription failure to retry subscribing settled invoices.
node.squeak_retention_s | int | [0,...] | yes | 604800 | SQUEAKNODE_NODE_SQUEAK_RETENTION_S | The amount of time in seconds to keep a squeak after download before deleting it. This only applies to squeaks that are not liked or created by a signing profile.
node.squeak_deletion_interval_s | int | [0,...] | yes | 10 | SQUEAKNODE_NODE_SQUEAK_DELETION_INTERVAL_S | The amount of time in seconds to wait in between deleting old squeaks.
node.offer_deletion_interval_s | int | [0,...] | yes | 10 | SQUEAKNODE_NODE_OFFER_DELETION_INTERVAL_S | The amount of time in seconds to wait in between deleting old offers.
node.interest_block_interval | int | [0,...] | yes | 2016 | SQUEAKNODE_NODE_INTEREST_BLOCK_INTERVAL | The number of blocks (starting from the most recent and descending) that this node will attempt to find squeaks with matching block height.
node.peer_autoconnect_interval_s | int | [0,...] | yes | 10 | SQUEAKNODE_NODE_PEER_AUTOCONNECT_INTERVAL_S | The amount of time in seconds to wait in between trying to connect autoconnect peers.
bitcoin.rpc_host | string | | yes | "localhost" | SQUEAKNODE_BITCOIN_RPC_HOST | The host of the bitcoin node to connect.
bitcoin.rpc_port | int | | yes | 18334 | SQUEAKNODE_BITCOIN_RPC_HOST | The port of the bitcoin node to connect.
bitcoin.rpc_user | string | | yes | "" | SQUEAKNODE_BITCOIN_RPC_USER | The username to use for authentication on the bitcoin node.
bitcoin.rpc_pass | string | | yes | "" | SQUEAKNODE_BITCOIN_RPC_PASS | The password to use for authentication on the bitcoin node.
bitcoin.rpc_use_ssl | boolean | [true, false] | yes | false | SQUEAKNODE_BITCOIN_USE_SSL | Use SSL for the connection to the bitcoin node.
bitcoin.rpc_ssl_cert | str |  | yes | "" | SQUEAKNODE_BITCOIN_SSL_CERT | The path to the SSL cert to use for connection to the bitcoin node, if one is used.
lightning.backend | string | | yes | "lnd" | SQUEAKNODE_LIGHTNING_BACKEND | The Lightning Network node implementation to use.
lightning.external_host | string | | yes | "" | SQUEAKNODE_LIGHTNING_EXTERNAL_HOST | The host of the lightning node to share with other peers.
lightning.external_port | int | | yes | | SQUEAKNODE_LIGHTNING_EXTERNAL_PORT | The port of the lightning node to share with other peers.
lightning.lnd_rpc_host | string | | yes | "localhost" | SQUEAKNODE_LIGHTNING_LND_RPC_HOST | The host of the LND node to connect.
lightning.lnd_rpc_port | int | | yes | | SQUEAKNODE_LIGHTNING_LND_RPC_PORT | The port of the LND node to use for RPC connections.
lightning.lnd_tls_cert_path | string | | yes | "" | SQUEAKNODE_LIGHTNING_LND_TLS_CERT_PATH | The path to the TLS certificate to use for LND connection.
lightning.lnd_macaroon_path | string | | yes | "" | SQUEAKNODE_LIGHTNING_LND_MACAROON_PATH | The path to the macaroon to use for LND connection.
lightning.clightning_rpc_file | string | | yes | "" | SQUEAKNODE_LIGHTNING_CLIGHTNING_RPC_FILE | The path to the file to use for c-lightning RPC.
tor.proxy_ip | string | | yes | "" | SQUEAKNODE_TOR_PROXY_IP | The ip address or host of the SOCKS5 Tor proxy, if one is used.
tor.proxy_port | int | | yes | 0 | SQUEAKNODE_TOR_PROXY_PORT | The port of the SOCKS5 Tor proxy, is one is used.
db.connection_string | string | | yes | "" | SQUEAKNODE_DB_CONNECTION_STRING | The connection string to use to connect to a SQL database. If none is specified, a sqlite database will be used on the local file system.
rpc.enabled | boolean | [true, false] | yes | false | SQUEAKNODE_RPC_ENABLED | Accept RPC commands or not.
rpc.host | string | | yes | "0.0.0.0" | SQUEAKNODE_RPC_HOST | Host to listen for rpc connections.
rpc.port | int | | yes | 8994 | SQUEAKNODE_RPC_PORT | Port to listen for rpc connections.
webadmin.enabled | boolean | [true, false] | yes | false | SQUEAKNODE_WEBADMIN_ENABLED | Run a web admin server or not.
webadmin.host | string | | yes | "0.0.0.0" | SQUEAKNODE_WEBADMIN_HOST | Host to user for serving admin web server.
webadmin.port | int | | yes | 12994 | SQUEAKNODE_WEBADMIN_PORT | Port to user for serving admin web server.
webadmin.username | string | | yes | "" | SQUEAKNODE_WEBADMIN_USERNAME | Username to sign in to admin web server.
webadmin.password | string | | yes | "" | SQUEAKNODE_WEBADMIN_PASSWORD | Password to sign in to admin web server.
webadmin.use_ssl | boolean | | yes | false | SQUEAKNODE_WEBADMIN_USE_SSL | Use SSL for admin web server or not.
webadmin.login_disabled | boolean | | yes | false | SQUEAKNODE_WEBADMIN_LOGIN_DISABLED | Disable requiring login for web server or not.
webadmin.allow_cors | boolean | | yes | false | SQUEAKNODE_WEBADMIN_ALLOW_CORS | Allow CORS requests to admin web server or not.
server.enabled | boolean | | yes | true | SQUEAKNODE_SERVER_ENABLED | If true, then accept inbound connections from other peers.
server.host | string | | yes | "0.0.0.0" | SQUEAKNODE_SERVER_HOST | Host to user for accepting inbound peer connections.
server.port | int | | yes | 8555/18555 | SQUEAKNODE_SERVER_PORT | Port to user for accepting inbound peer connections.
server.external_address | string | | yes | "" | SQUEAKNODE_SERVER_EXTERNAL_ADDRESS | The address that other nodes should use to open a connection to this node.
